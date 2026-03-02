use super::markdown_config::{
    MarkdownConfigMemoryIndex, extract_markdown_config_blocks,
    extract_markdown_config_link_targets_by_id,
};
use crate::WendaoResourceUri;
use include_dir::Dir;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

fn is_markdown_file(path: &str) -> bool {
    matches!(
        path.rsplit('.').next().map(str::to_ascii_lowercase),
        Some(ext) if ext == "md" || ext == "markdown"
    )
}

fn normalize_registry_key(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}

fn is_wendao_uri(target: &str) -> bool {
    WendaoResourceUri::parse(target).is_ok()
}

/// One normalized config link target with optional type-hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WendaoResourceLinkTarget {
    /// Normalized target path or semantic URI.
    pub target_path: String,
    /// Optional link type-hint (for example `template`, `persona`).
    pub reference_type: Option<String>,
}

fn collect_embedded_markdown_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a include_dir::File<'a>>) {
    for file in dir.files() {
        let path = file.path().to_string_lossy().replace('\\', "/");
        if is_markdown_file(path.as_str()) {
            out.push(file);
        }
    }
    for child in dir.dirs() {
        collect_embedded_markdown_files(child, out);
    }
}

/// One unresolved link edge found during embedded-registry validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingEmbeddedLink {
    /// Markdown source file path inside embedded resources.
    pub source_path: String,
    /// Config block id owning this link scope.
    pub id: String,
    /// Linked target path that was not found.
    pub target_path: String,
}

/// Error type for `WendaoResourceRegistry::build_from_embedded`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WendaoResourceRegistryError {
    /// Embedded markdown file bytes could not be decoded as UTF-8 text.
    #[error("embedded markdown file is not valid UTF-8: {path}")]
    InvalidUtf8 {
        /// Failing markdown file path inside embedded resources.
        path: String,
    },
    /// One or more link targets declared in markdown could not be resolved.
    #[error("embedded markdown registry found {count} missing linked resource(s)")]
    MissingLinkedResources {
        /// Number of unresolved links.
        count: usize,
        /// Detailed unresolved links.
        missing: Vec<MissingEmbeddedLink>,
    },
}

/// Per-file view for markdown config links extracted from embedded resources.
#[derive(Debug, Clone, Default)]
pub struct WendaoResourceFile {
    path: String,
    links_by_id: HashMap<String, Vec<String>>,
    link_targets_by_id: HashMap<String, Vec<WendaoResourceLinkTarget>>,
}

impl WendaoResourceFile {
    /// File path (relative to embedded resources root).
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns all linked targets for one config `id`.
    #[must_use]
    pub fn links_for_id(&self, id: &str) -> Option<&[String]> {
        self.links_by_id.get(id).map(Vec::as_slice)
    }

    /// Full map of `id -> linked targets` parsed from this file.
    #[must_use]
    pub fn links_by_id(&self) -> &HashMap<String, Vec<String>> {
        &self.links_by_id
    }

    /// Full map of `id -> link targets` with optional type-hints.
    #[must_use]
    pub fn link_targets_by_id(&self) -> &HashMap<String, Vec<WendaoResourceLinkTarget>> {
        &self.link_targets_by_id
    }

    /// Returns link targets for one config `id` including optional type-hints.
    #[must_use]
    pub fn link_targets_for_id(&self, id: &str) -> Option<&[WendaoResourceLinkTarget]> {
        self.link_targets_by_id.get(id).map(Vec::as_slice)
    }

    /// Resolves deduplicated semantic links for one reference type.
    ///
    /// Type matching is ASCII case-insensitive and uses wikilink suffixes
    /// such as `#persona`, `#template`, `#knowledge`, or `#qianji-flow`.
    #[must_use]
    pub fn links_for_reference_type(&self, reference_type: &str) -> Vec<String> {
        let normalized_type = reference_type.trim().to_ascii_lowercase();
        if normalized_type.is_empty() {
            return Vec::new();
        }

        let mut links = self
            .link_targets_by_id
            .values()
            .flatten()
            .filter(|target| target.reference_type.as_deref() == Some(normalized_type.as_str()))
            .map(|target| target.target_path.clone())
            .collect::<Vec<_>>();
        links.sort();
        links.dedup();
        links
    }
}

/// Embedded markdown registry parsed by Wendao AST utilities.
#[derive(Debug, Clone, Default)]
pub struct WendaoResourceRegistry {
    files_by_path: HashMap<String, WendaoResourceFile>,
    config_index: MarkdownConfigMemoryIndex,
}

impl WendaoResourceRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a registry from compile-time embedded resources.
    ///
    /// This scans markdown files, indexes tagged config blocks in O(1) by `id`,
    /// extracts local linked targets per `id`, and validates each linked target
    /// exists in the same embedded directory tree.
    ///
    /// # Errors
    ///
    /// Returns [`WendaoResourceRegistryError::InvalidUtf8`] when an embedded
    /// markdown file cannot be decoded as UTF-8.
    ///
    /// Returns [`WendaoResourceRegistryError::MissingLinkedResources`] when
    /// markdown links reference files missing from embedded resources.
    pub fn build_from_embedded(embedded: &Dir<'_>) -> Result<Self, WendaoResourceRegistryError> {
        let mut registry = Self::new();
        let mut markdown_files = Vec::new();
        collect_embedded_markdown_files(embedded, &mut markdown_files);
        markdown_files.sort_by(|left, right| left.path().cmp(right.path()));

        let mut missing_links: Vec<MissingEmbeddedLink> = Vec::new();

        for file in markdown_files {
            let relative_path = normalize_registry_key(file.path().to_string_lossy().as_ref());
            let Some(markdown) = file.contents_utf8() else {
                return Err(WendaoResourceRegistryError::InvalidUtf8 {
                    path: relative_path,
                });
            };

            registry
                .config_index
                .extend(extract_markdown_config_blocks(markdown));

            let semantic_skill_name =
                semantic_skill_name_from_descriptor(relative_path.as_str(), markdown);
            let raw_link_targets =
                extract_markdown_config_link_targets_by_id(markdown, &relative_path);
            let link_targets_by_id = raw_link_targets
                .iter()
                .map(|(id, targets)| {
                    (
                        id.clone(),
                        targets
                            .iter()
                            .map(|target| WendaoResourceLinkTarget {
                                target_path: semantic_lift_target(
                                    target.target.as_str(),
                                    relative_path.as_str(),
                                    semantic_skill_name.as_deref(),
                                ),
                                reference_type: target.reference_type.clone(),
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<HashMap<_, _>>();
            let links_by_id = raw_link_targets
                .iter()
                .map(|(id, targets)| {
                    (
                        id.clone(),
                        targets
                            .iter()
                            .map(|target| {
                                semantic_lift_target(
                                    target.target.as_str(),
                                    relative_path.as_str(),
                                    semantic_skill_name.as_deref(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<HashMap<_, _>>();

            for (id, targets) in &raw_link_targets {
                for target in targets {
                    if is_wendao_uri(target.target.as_str()) {
                        continue;
                    }
                    if embedded.get_file(target.target.as_str()).is_none() {
                        missing_links.push(MissingEmbeddedLink {
                            source_path: relative_path.clone(),
                            id: id.clone(),
                            target_path: target.target.clone(),
                        });
                    }
                }
            }

            registry.files_by_path.insert(
                relative_path.clone(),
                WendaoResourceFile {
                    path: relative_path,
                    links_by_id,
                    link_targets_by_id,
                },
            );
        }

        if missing_links.is_empty() {
            Ok(registry)
        } else {
            Err(WendaoResourceRegistryError::MissingLinkedResources {
                count: missing_links.len(),
                missing: missing_links,
            })
        }
    }

    /// O(1) config block lookup by exact `id`.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&super::markdown_config::MarkdownConfigBlock> {
        self.config_index.get(id)
    }

    /// Returns one embedded markdown file entry by relative path.
    #[must_use]
    pub fn file(&self, path: &str) -> Option<&WendaoResourceFile> {
        self.files_by_path.get(&normalize_registry_key(path))
    }

    /// Iterates all embedded markdown file entries.
    pub fn files(&self) -> impl Iterator<Item = &WendaoResourceFile> {
        self.files_by_path.values()
    }

    /// Number of indexed markdown files.
    #[must_use]
    pub fn files_len(&self) -> usize {
        self.files_by_path.len()
    }

    /// Access to the underlying O(1) markdown config index.
    #[must_use]
    pub fn config_index(&self) -> &MarkdownConfigMemoryIndex {
        &self.config_index
    }
}

fn semantic_skill_name_from_descriptor(path: &str, markdown: &str) -> Option<String> {
    if !Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md") || name == "skill.md")
    {
        return None;
    }
    crate::parse_frontmatter(markdown)
        .name
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

fn semantic_lift_target(
    target: &str,
    source_path: &str,
    semantic_skill_name: Option<&str>,
) -> String {
    if is_wendao_uri(target) {
        return target.to_string();
    }
    let Some(semantic_skill_name) = semantic_skill_name else {
        return target.to_string();
    };

    let source_parent = Path::new(source_path).parent();
    let Some(source_parent) = source_parent else {
        return target.to_string();
    };
    let references_dir = source_parent.join("references");
    let target_path = Path::new(target);
    let Ok(relative_entity) = target_path.strip_prefix(references_dir.as_path()) else {
        return target.to_string();
    };
    let normalized_entity = normalize_registry_key(relative_entity.to_string_lossy().as_ref());
    if normalized_entity.is_empty() {
        return target.to_string();
    }
    format!("wendao://skills/{semantic_skill_name}/references/{normalized_entity}")
}
