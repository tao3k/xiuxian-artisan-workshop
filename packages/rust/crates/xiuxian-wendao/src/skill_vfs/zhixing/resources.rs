//! Embedded `zhixing` resource accessors backed by Wendao AST parsing.

use crate::{KnowledgeGraph, WendaoResourceRegistry, WendaoResourceUri, parse_frontmatter};
use include_dir::{Dir, include_dir};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use crate::skill_vfs::zhixing::{Error, Result};

/// Embedded skill document path relative to the `resources/` root.
pub const ZHIXING_SKILL_DOC_PATH: &str = "zhixing/skills/agenda-management/SKILL.md";
pub(crate) const ZHIXING_EMBEDDED_CRATE_ID: &str = "xiuxian-zhixing";

static EMBEDDED_ZHIXING_RESOURCES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../xiuxian-zhixing/resources");
static EMBEDDED_MOUNTS_BY_SEMANTIC: OnceLock<HashMap<String, Vec<PathBuf>>> = OnceLock::new();
static EMBEDDED_DISCOVERY_RECORDS: OnceLock<
    std::result::Result<Vec<EmbeddedDiscoveryRecord>, String>,
> = OnceLock::new();

#[derive(Debug, Clone, Default)]
struct EmbeddedDiscoveryRecord {
    uri: String,
    reference_ids: HashSet<String>,
    reference_types: HashSet<String>,
    search_blob: String,
}

#[must_use]
pub(crate) fn embedded_resource_dir() -> &'static Dir<'static> {
    &EMBEDDED_ZHIXING_RESOURCES
}

#[must_use]
pub(crate) fn embedded_semantic_reference_mounts() -> &'static HashMap<String, Vec<PathBuf>> {
    embedded_skill_mount_index()
}

fn normalize_embedded_resource_path(path: &str) -> String {
    path.trim().trim_start_matches("./").replace('\\', "/")
}

/// Returns the embedded markdown source of `zhixing/skills/agenda-management/SKILL.md`.
#[must_use]
pub fn embedded_skill_markdown() -> Option<&'static str> {
    embedded_resource_text(ZHIXING_SKILL_DOC_PATH)
}

/// Returns UTF-8 text content for one embedded resource path.
///
/// Paths are normalized to slash separators and accept optional `./` prefix.
#[must_use]
pub fn embedded_resource_text(path: &str) -> Option<&'static str> {
    let normalized = normalize_embedded_resource_path(path);
    EMBEDDED_ZHIXING_RESOURCES
        .get_file(normalized.as_str())
        .and_then(include_dir::File::contents_utf8)
}

/// Resolves one semantic `wendao://` URI from embedded zhixing resources.
///
/// This API is intentionally strict: only semantic URIs are supported.
#[must_use]
pub fn embedded_resource_text_from_wendao_uri(uri: &str) -> Option<&'static str> {
    let parsed = WendaoResourceUri::parse(uri).ok()?;
    embedded_resource_text_from_parsed_wendao_uri(&parsed)
}

/// Resolves one parsed semantic URI from embedded Zhixing resources.
#[must_use]
fn embedded_resource_text_from_parsed_wendao_uri(uri: &WendaoResourceUri) -> Option<&'static str> {
    let mounts = embedded_skill_mount_index().get(uri.semantic_name())?;
    let candidate = uri.entity_relative_path();
    for mount in mounts {
        let target =
            normalize_embedded_resource_path(mount.join(candidate).to_string_lossy().as_ref());
        let Some(content) = embedded_resource_text(target.as_str()) else {
            continue;
        };
        return Some(content);
    }
    None
}

fn embedded_skill_mount_index() -> &'static HashMap<String, Vec<PathBuf>> {
    EMBEDDED_MOUNTS_BY_SEMANTIC.get_or_init(resolve_embedded_skill_mount_index)
}

fn resolve_embedded_skill_mount_index() -> HashMap<String, Vec<PathBuf>> {
    let mut markdown_files = Vec::new();
    collect_embedded_markdown_files(&EMBEDDED_ZHIXING_RESOURCES, &mut markdown_files);
    markdown_files.sort_by(|left, right| left.path().cmp(right.path()));

    let mut mounts_by_semantic: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for file in markdown_files {
        let path = normalize_embedded_resource_path(file.path().to_string_lossy().as_ref());
        if !is_skill_descriptor(path.as_str()) {
            continue;
        }
        let Some(content) = file.contents_utf8() else {
            continue;
        };
        let semantic_name = parse_frontmatter(content)
            .name
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());
        let Some(semantic_name) = semantic_name else {
            continue;
        };

        let references_dir = Path::new(path.as_str()).parent().map_or_else(
            || PathBuf::from("references"),
            |parent| parent.join("references"),
        );
        mounts_by_semantic
            .entry(semantic_name)
            .or_default()
            .push(references_dir);
    }

    for references_dirs in mounts_by_semantic.values_mut() {
        references_dirs.sort();
        references_dirs.dedup();
    }

    mounts_by_semantic
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

fn is_markdown_file(path: &str) -> bool {
    matches!(
        path.rsplit('.').next().map(str::to_ascii_lowercase),
        Some(ext) if ext == "md" || ext == "markdown"
    )
}

fn is_skill_descriptor(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md") || name == "skill.md")
}

/// Builds Wendao AST registry from embedded `xiuxian-zhixing/resources`.
///
/// # Errors
///
/// Returns an error when embedded markdown parsing fails or when linked
/// resource targets in markdown cannot be resolved from embedded files.
pub fn build_embedded_wendao_registry() -> Result<WendaoResourceRegistry> {
    WendaoResourceRegistry::build_from_embedded(&EMBEDDED_ZHIXING_RESOURCES).map_err(|error| {
        Error::Internal(format!("failed to build embedded wendao registry: {error}"))
    })
}

/// Resolves linked resource paths under the `zhixing/skills/agenda-management/SKILL.md` section for one id.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_skill_links_for_id(id: &str) -> Result<Vec<String>> {
    let links_by_id = embedded_skill_links_index()?;
    if let Some(links) = links_by_id.get(id) {
        return Ok(links.clone());
    }

    // Multi-skill fallback: scan all embedded SKILL.md registries for the same id.
    let registry = build_embedded_wendao_registry()?;
    let mut links = registry
        .files()
        .filter_map(|file| file.links_for_id(id))
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    links.sort();
    links.dedup();
    Ok(links)
}

/// Resolves linked semantic URIs under `zhixing/skills/agenda-management/SKILL.md` for one reference type.
///
/// Type matching is ASCII case-insensitive and based on wikilink type-hints
/// such as `#persona`, `#template`, `#knowledge`, and `#qianji-flow`.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_skill_links_for_reference_type(reference_type: &str) -> Result<Vec<String>> {
    let registry = build_embedded_wendao_registry()?;
    let Some(skill_file) = registry.file(ZHIXING_SKILL_DOC_PATH) else {
        return Ok(Vec::new());
    };
    Ok(skill_file.links_for_reference_type(reference_type))
}

/// Returns all parsed linked resource paths keyed by heading `id` in `zhixing/skills/agenda-management/SKILL.md`.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_skill_links_index() -> Result<HashMap<String, Vec<String>>> {
    let registry = build_embedded_wendao_registry()?;
    Ok(registry
        .file(ZHIXING_SKILL_DOC_PATH)
        .map_or_else(HashMap::new, |entry| entry.links_by_id().clone()))
}

/// Discovers canonical semantic URIs from one runtime query expression.
///
/// Supported query forms:
/// - `reference_type:<type>` (or `type:<type>`, `ref_type:<type>`)
/// - `id:<config_id>`
/// - free semantic query (for example `carryover:>=1`)
///
/// Free semantic queries perform token matching across canonical URI, markdown
/// content, and frontmatter-derived hints.
///
/// # Errors
///
/// Returns an error when embedded registry construction fails.
pub fn embedded_discover_canonical_uris(query: &str) -> Result<Vec<String>> {
    let normalized_query = query.trim();
    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }
    let normalized_query = normalized_query
        .strip_prefix("query:")
        .map_or(normalized_query, str::trim);
    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }

    let records = embedded_discovery_records()?;

    if let Some(reference_type) =
        parse_prefixed_value(normalized_query, &["reference_type", "type", "ref_type"])
    {
        let hits = discover_by_reference_type(records, reference_type);
        if !hits.is_empty() {
            return Ok(hits);
        }
        return embedded_skill_links_for_reference_type(reference_type);
    }
    if let Some(config_id) = parse_prefixed_value(normalized_query, &["id"]) {
        let hits = discover_by_config_id(records, config_id);
        if !hits.is_empty() {
            return Ok(hits);
        }
        return embedded_skill_links_for_id(config_id);
    }

    let terms = semantic_query_terms(normalized_query);
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let hits = discover_by_semantic_terms(records, terms.as_slice());
    if !hits.is_empty() {
        return Ok(hits);
    }

    // Conservative fallback keeps behavior stable even if the graph-backed
    // discovery cache is temporarily incomplete.
    discover_by_registry_scan(terms.as_slice())
}

fn parse_prefixed_value<'a>(query: &'a str, keys: &[&str]) -> Option<&'a str> {
    let lowered = query.to_ascii_lowercase();
    for key in keys {
        let prefix = format!("{key}:");
        if lowered.starts_with(prefix.as_str()) {
            let value = query[prefix.len()..].trim();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

fn semantic_query_terms(query: &str) -> Vec<String> {
    let mut terms = query
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_')
        .map(str::trim)
        .filter(|term| term.len() >= 2)
        .filter(|term| term.chars().any(|ch| !ch.is_ascii_digit()))
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn embedded_discovery_records() -> Result<&'static [EmbeddedDiscoveryRecord]> {
    match EMBEDDED_DISCOVERY_RECORDS.get_or_init(build_embedded_discovery_records) {
        Ok(records) => Ok(records.as_slice()),
        Err(reason) => Err(Error::Internal(format!(
            "failed to build embedded discovery graph cache: {reason}"
        ))),
    }
}

fn build_embedded_discovery_records() -> std::result::Result<Vec<EmbeddedDiscoveryRecord>, String> {
    let graph = Arc::new(KnowledgeGraph::new());
    let indexer = super::indexer::ZhixingWendaoIndexer::new(Arc::clone(&graph), PathBuf::new());
    indexer
        .index_embedded_skill_references_only()
        .map_err(|error| error.to_string())?;

    let mut by_uri: HashMap<String, EmbeddedDiscoveryRecord> = HashMap::new();
    for relation in graph.get_all_relations() {
        let Some(uri) = relation
            .metadata
            .get("reference_uri")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let Ok(parsed_uri) = WendaoResourceUri::parse(uri) else {
            continue;
        };
        let canonical_uri = parsed_uri.canonical_uri();
        let record =
            by_uri
                .entry(canonical_uri.clone())
                .or_insert_with(|| EmbeddedDiscoveryRecord {
                    uri: canonical_uri,
                    ..EmbeddedDiscoveryRecord::default()
                });
        if let Some(reference_id) = relation
            .metadata
            .get("reference_id")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            record
                .reference_ids
                .insert(reference_id.to_ascii_lowercase());
        }
        if let Some(reference_type) = relation
            .metadata
            .get("reference_type")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            record
                .reference_types
                .insert(reference_type.to_ascii_lowercase());
        }
    }

    for record in by_uri.values_mut() {
        let mut haystack = String::new();
        haystack.push_str(record.uri.to_ascii_lowercase().as_str());
        for reference_id in &record.reference_ids {
            haystack.push('\n');
            haystack.push_str(reference_id.as_str());
        }
        for reference_type in &record.reference_types {
            haystack.push('\n');
            haystack.push_str(reference_type.as_str());
        }

        if let Some(content) = embedded_resource_text_from_wendao_uri(record.uri.as_str()) {
            haystack.push('\n');
            haystack.push_str(content.to_ascii_lowercase().as_str());
            let frontmatter = parse_frontmatter(content);
            if let Some(name) = frontmatter.name.as_deref() {
                haystack.push('\n');
                haystack.push_str(name.to_ascii_lowercase().as_str());
            }
            for keyword in frontmatter.routing_keywords {
                haystack.push('\n');
                haystack.push_str(keyword.to_ascii_lowercase().as_str());
            }
            for intent in frontmatter.intents {
                haystack.push('\n');
                haystack.push_str(intent.to_ascii_lowercase().as_str());
            }
        }
        record.search_blob = haystack;
    }

    let mut records = by_uri.into_values().collect::<Vec<_>>();
    records.sort_by(|left, right| left.uri.cmp(&right.uri));
    Ok(records)
}

fn discover_by_reference_type(
    records: &[EmbeddedDiscoveryRecord],
    reference_type: &str,
) -> Vec<String> {
    let normalized = reference_type.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut hits = records
        .iter()
        .filter(|record| record.reference_types.contains(normalized.as_str()))
        .map(|record| record.uri.clone())
        .collect::<Vec<_>>();
    hits.sort();
    hits.dedup();
    hits
}

fn discover_by_config_id(records: &[EmbeddedDiscoveryRecord], config_id: &str) -> Vec<String> {
    let normalized = config_id.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut hits = records
        .iter()
        .filter(|record| record.reference_ids.contains(normalized.as_str()))
        .map(|record| record.uri.clone())
        .collect::<Vec<_>>();
    hits.sort();
    hits.dedup();
    hits
}

fn discover_by_semantic_terms(
    records: &[EmbeddedDiscoveryRecord],
    terms: &[String],
) -> Vec<String> {
    let mut hits = records
        .iter()
        .filter(|record| {
            terms
                .iter()
                .all(|term| record.search_blob.contains(term.as_str()))
        })
        .map(|record| record.uri.clone())
        .collect::<Vec<_>>();
    hits.sort();
    hits.dedup();
    hits
}

fn discover_by_registry_scan(terms: &[String]) -> Result<Vec<String>> {
    let mut candidates = embedded_skill_links_index()?
        .into_values()
        .flatten()
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();

    let mut hits = Vec::new();
    for uri in candidates {
        let Some(content) = embedded_resource_text_from_wendao_uri(uri.as_str()) else {
            continue;
        };
        let frontmatter = parse_frontmatter(content);
        let mut haystacks = Vec::with_capacity(5);
        haystacks.push(uri.to_ascii_lowercase());
        haystacks.push(content.to_ascii_lowercase());
        if let Some(name) = frontmatter.name.as_deref() {
            haystacks.push(name.to_ascii_lowercase());
        }
        haystacks.extend(
            frontmatter
                .routing_keywords
                .iter()
                .map(|value| value.to_ascii_lowercase()),
        );
        haystacks.extend(
            frontmatter
                .intents
                .iter()
                .map(|value| value.to_ascii_lowercase()),
        );
        if terms
            .iter()
            .all(|term| haystacks.iter().any(|entry| entry.contains(term.as_str())))
        {
            hits.push(uri);
        }
    }
    hits.sort();
    hits.dedup();
    Ok(hits)
}
