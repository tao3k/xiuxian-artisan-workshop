use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue},
    parse_document,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

const DEFAULT_CONFIG_TYPE: &str = "unknown";
const TEMPLATE_CONFIG_TYPE: &str = "template";
const PERSONA_CONFIG_TYPE: &str = "persona";
const CONFIG_ID_KEY: &str = "id";
const CONFIG_TYPE_KEY: &str = "type";
const CONFIG_TARGET_KEY: &str = "target";

/// Extracted markdown configuration block bound to a tagged heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownConfigBlock {
    /// Exact identifier from HTML property tag.
    pub id: String,
    /// Configuration kind from HTML property tag.
    pub config_type: String,
    /// Optional logical template target.
    pub target: Option<String>,
    /// Heading title that owns this config block.
    pub heading: String,
    /// Fenced code language (for example `jinja2`).
    pub language: String,
    /// Raw code block content extracted from AST.
    pub content: String,
}

/// O(1) in-memory index for markdown configuration blocks keyed by `id`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkdownConfigMemoryIndex {
    blocks_by_id: HashMap<String, MarkdownConfigBlock>,
}

impl MarkdownConfigMemoryIndex {
    /// Creates an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds an index from markdown by parsing tagged AST blocks.
    #[must_use]
    pub fn from_markdown(markdown: &str) -> Self {
        Self::from_blocks(extract_markdown_config_blocks(markdown))
    }

    /// Builds an index from pre-extracted blocks.
    #[must_use]
    pub fn from_blocks<I>(blocks: I) -> Self
    where
        I: IntoIterator<Item = MarkdownConfigBlock>,
    {
        let mut index = Self::new();
        index.extend(blocks);
        index
    }

    /// Inserts or replaces one block by its exact `id`.
    pub fn insert(&mut self, block: MarkdownConfigBlock) -> Option<MarkdownConfigBlock> {
        self.blocks_by_id.insert(block.id.clone(), block)
    }

    /// Extends the index with multiple blocks.
    pub fn extend<I>(&mut self, blocks: I)
    where
        I: IntoIterator<Item = MarkdownConfigBlock>,
    {
        for block in blocks {
            self.insert(block);
        }
    }

    /// Returns a block by exact `id` lookup in O(1).
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&MarkdownConfigBlock> {
        self.blocks_by_id.get(id)
    }

    /// Returns the number of indexed blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks_by_id.len()
    }

    /// Returns `true` when the index has no blocks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks_by_id.is_empty()
    }

    /// Returns an iterator over all indexed config blocks.
    pub fn values(&self) -> impl Iterator<Item = &MarkdownConfigBlock> {
        self.blocks_by_id.values()
    }
}

/// Extracts tagged configuration code blocks from markdown AST.
///
/// The parser scans heading nodes, checks their adjacent HTML property blocks,
/// keeps that property as a cursor, and extracts fenced `jinja2` code blocks
/// under the active heading scope.
#[must_use]
pub fn extract_markdown_config_blocks(markdown: &str) -> Vec<MarkdownConfigBlock> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());

    let mut extracted: Vec<MarkdownConfigBlock> = Vec::new();
    let mut active_cursor: Option<MarkdownPropertyCursor> = None;

    for node in root.descendants() {
        match &node.data.borrow().value {
            NodeValue::Heading(heading) => {
                let heading_level = heading.level;
                if let Some(cursor) = &active_cursor
                    && heading_level <= cursor.heading_level
                {
                    active_cursor = None;
                }
                if let Some(next_cursor) = parse_cursor_from_heading(node, heading_level) {
                    active_cursor = Some(next_cursor);
                }
            }
            NodeValue::CodeBlock(block) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                let Some(language) = parse_fence_language(&block.info) else {
                    continue;
                };
                if !is_extractable_config_code_block(&cursor.config_type, &language) {
                    continue;
                }
                extracted.push(MarkdownConfigBlock {
                    id: cursor.id.clone(),
                    config_type: cursor.config_type.clone(),
                    target: cursor.target.clone(),
                    heading: cursor.heading.clone(),
                    language,
                    content: block.literal.clone(),
                });
            }
            _ => {}
        }
    }

    extracted
}

/// One normalized link target extracted under a tagged config heading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownConfigLinkTarget {
    /// Normalized target path or semantic URI.
    pub target: String,
    /// Optional type-hint parsed from a wikilink suffix (for example `#persona`).
    pub reference_type: Option<String>,
}

/// Extracts local markdown/wikilink targets under each tagged heading scope.
///
/// Returned map shape:
/// - key: config `id` from `<!-- id: "...", type: "..." -->`
/// - value: normalized relative resource paths linked from that scope
///
/// External links (`https://...`, `mailto:...`) and pure fragments (`#section`)
/// are ignored.
#[must_use]
pub fn extract_markdown_config_links_by_id(
    markdown: &str,
    source_path: &str,
) -> HashMap<String, Vec<String>> {
    extract_markdown_config_link_targets_by_id(markdown, source_path)
        .into_iter()
        .map(|(id, targets)| {
            let paths = targets
                .into_iter()
                .map(|target| target.target)
                .collect::<Vec<_>>();
            (id, paths)
        })
        .collect()
}

/// Extracts normalized local/semantic link targets plus optional type-hints.
///
/// Type hints are parsed from wikilink shape `[[target#type]]`.
#[must_use]
pub fn extract_markdown_config_link_targets_by_id(
    markdown: &str,
    source_path: &str,
) -> HashMap<String, Vec<MarkdownConfigLinkTarget>> {
    let mut options = Options::default();
    options.extension.wikilinks_title_before_pipe = true;

    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options);

    let mut links_by_id: HashMap<String, Vec<MarkdownConfigLinkTarget>> = HashMap::new();
    let mut active_cursor: Option<MarkdownPropertyCursor> = None;

    for node in root.descendants() {
        match &node.data.borrow().value {
            NodeValue::Heading(heading) => {
                let heading_level = heading.level;
                if let Some(cursor) = &active_cursor
                    && heading_level <= cursor.heading_level
                {
                    active_cursor = None;
                }
                if let Some(next_cursor) = parse_cursor_from_heading(node, heading_level) {
                    active_cursor = Some(next_cursor);
                }
            }
            NodeValue::Link(link) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                insert_link_target(
                    &mut links_by_id,
                    &cursor.id,
                    link.url.as_str(),
                    source_path,
                    None,
                );
            }
            NodeValue::Image(image) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                insert_link_target(
                    &mut links_by_id,
                    &cursor.id,
                    image.url.as_str(),
                    source_path,
                    Some("attachment".to_string()),
                );
            }
            NodeValue::WikiLink(link) => {
                let Some(cursor) = &active_cursor else {
                    continue;
                };
                let (target, reference_type) = split_wikilink_type_hint(link.url.as_str());
                insert_link_target(
                    &mut links_by_id,
                    &cursor.id,
                    target,
                    source_path,
                    reference_type,
                );
            }
            _ => {}
        }
    }

    links_by_id
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownPropertyCursor {
    id: String,
    config_type: String,
    target: Option<String>,
    heading: String,
    heading_level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownPropertyTag {
    id: String,
    config_type: String,
    target: Option<String>,
}

fn parse_cursor_from_heading<'a>(
    heading_node: &'a AstNode<'a>,
    heading_level: u8,
) -> Option<MarkdownPropertyCursor> {
    let heading = collect_heading_text(heading_node);
    let sibling = heading_node.next_sibling()?;
    let NodeValue::HtmlBlock(html) = &sibling.data.borrow().value else {
        return None;
    };
    let tag = parse_property_tag(&html.literal)?;
    Some(MarkdownPropertyCursor {
        id: tag.id,
        config_type: tag.config_type,
        target: tag.target,
        heading,
        heading_level,
    })
}

fn collect_heading_text<'a>(heading_node: &'a AstNode<'a>) -> String {
    let mut heading = String::new();
    for child in heading_node.children() {
        push_text_from_node(child, &mut heading);
    }
    heading.trim().to_string()
}

fn push_text_from_node<'a>(node: &'a AstNode<'a>, out: &mut String) {
    match &node.data.borrow().value {
        NodeValue::Text(value) => out.push_str(value),
        NodeValue::Code(value) => out.push_str(&value.literal),
        NodeValue::SoftBreak | NodeValue::LineBreak => out.push(' '),
        _ => {
            for child in node.children() {
                push_text_from_node(child, out);
            }
        }
    }
}

fn parse_property_tag(html_block: &str) -> Option<MarkdownPropertyTag> {
    let body = html_block
        .trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")?
        .trim();

    let mut id: Option<String> = None;
    let mut config_type: Option<String> = None;
    let mut target: Option<String> = None;

    for pair in body.split(',') {
        let Some((raw_key, raw_value)) = pair.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let value = trim_quotes(raw_value.trim());
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            CONFIG_ID_KEY => id = Some(value.to_string()),
            CONFIG_TYPE_KEY => config_type = Some(value.to_string()),
            CONFIG_TARGET_KEY => target = Some(value.to_string()),
            _ => {}
        }
    }

    Some(MarkdownPropertyTag {
        id: id?,
        config_type: config_type.unwrap_or_else(|| DEFAULT_CONFIG_TYPE.to_string()),
        target,
    })
}

fn trim_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn parse_fence_language(info: &str) -> Option<String> {
    info.split_whitespace().next().map(str::to_lowercase)
}

fn is_jinja2_fence(language: &str) -> bool {
    language == "jinja2" || language == "j2"
}

fn is_toml_fence(language: &str) -> bool {
    language == "toml"
}

fn is_extractable_config_code_block(config_type: &str, language: &str) -> bool {
    match config_type.trim().to_ascii_lowercase().as_str() {
        TEMPLATE_CONFIG_TYPE => is_jinja2_fence(language),
        PERSONA_CONFIG_TYPE => is_toml_fence(language),
        _ => false,
    }
}

fn insert_link_target(
    links_by_id: &mut HashMap<String, Vec<MarkdownConfigLinkTarget>>,
    id: &str,
    raw_target: &str,
    source_path: &str,
    reference_type: Option<String>,
) {
    let Some(target) = normalize_local_link_target(raw_target, source_path) else {
        return;
    };
    let normalized_type = normalize_reference_type(reference_type, target.as_str());
    let links = links_by_id.entry(id.to_string()).or_default();
    if !links
        .iter()
        .any(|existing| existing.target == target && existing.reference_type == normalized_type)
    {
        links.push(MarkdownConfigLinkTarget {
            target,
            reference_type: normalized_type,
        });
    }
}

fn normalize_reference_type(reference_type: Option<String>, target: &str) -> Option<String> {
    let explicit = reference_type
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    explicit.or_else(|| infer_reference_type_from_target(target))
}

fn infer_reference_type_from_target(target: &str) -> Option<String> {
    let ext = extract_extension(target)?;
    if is_attachment_extension(ext) {
        return Some("attachment".to_string());
    }
    None
}

fn extract_extension(target: &str) -> Option<&str> {
    let without_fragment = strip_fragment_and_query(target);
    let leaf = without_fragment.rsplit('/').next()?;
    let (_, extension) = leaf.rsplit_once('.')?;
    let trimmed = extension.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn is_attachment_extension(extension: &str) -> bool {
    matches!(
        extension.trim().to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "pdf"
    )
}

fn split_wikilink_type_hint(raw_target: &str) -> (&str, Option<String>) {
    let trimmed = raw_target.trim();
    if trimmed.is_empty() {
        return (trimmed, None);
    }
    let before_alias = trimmed.split('|').next().unwrap_or(trimmed).trim();
    let Some((target, hint)) = before_alias.rsplit_once('#') else {
        return (before_alias, None);
    };
    let hint = hint.trim();
    if target.trim().is_empty() || hint.is_empty() {
        return (before_alias, None);
    }
    (target.trim(), Some(hint.to_string()))
}

fn normalize_local_link_target(raw_target: &str, source_path: &str) -> Option<String> {
    let target = strip_fragment_and_query(raw_target);
    if target.is_empty() || target.starts_with('#') {
        return None;
    }
    if is_wendao_resource_uri(target) {
        return Some(target.to_string());
    }
    if is_external_target(target) {
        return None;
    }

    let source_parent = Path::new(source_path).parent().unwrap_or(Path::new(""));
    let target_path = if target.starts_with('/') {
        PathBuf::from(target.trim_start_matches('/'))
    } else {
        source_parent.join(target)
    };
    normalize_relative_path(&target_path)
}

fn normalize_relative_path(path: &Path) -> Option<String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(value) => normalized.push(value),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    let candidate = normalized.to_string_lossy().replace('\\', "/");
    if candidate.is_empty() {
        None
    } else {
        Some(candidate)
    }
}

fn strip_fragment_and_query(raw: &str) -> &str {
    let mut end = raw.len();
    if let Some(index) = raw.find('#') {
        end = end.min(index);
    }
    if let Some(index) = raw.find('?') {
        end = end.min(index);
    }
    raw[..end].trim()
}

fn is_external_target(target: &str) -> bool {
    target.contains("://") || target.starts_with("mailto:") || target.starts_with("tel:")
}

fn is_wendao_resource_uri(target: &str) -> bool {
    target.trim().to_ascii_lowercase().starts_with("wendao://")
}
