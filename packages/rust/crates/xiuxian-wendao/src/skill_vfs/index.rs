use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use xiuxian_skills::{SkillScanner, parse_frontmatter_from_markdown};

use super::SkillVfsError;

/// One mounted semantic namespace in skill VFS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillNamespaceMount {
    /// Semantic namespace from SKILL frontmatter `name`.
    pub semantic_name: String,
    /// Descriptor path that declared the namespace.
    pub skill_doc: PathBuf,
    /// Relative resource root (`references/`) for this namespace.
    pub references_dir: PathBuf,
}

/// In-memory semantic namespace index built from skill roots.
#[derive(Debug, Clone, Default)]
pub struct SkillNamespaceIndex {
    mounts_by_name: HashMap<String, Vec<SkillNamespaceMount>>,
    paths_by_uri: HashMap<String, PathBuf>,
}

impl SkillNamespaceIndex {
    /// Build namespace index by scanning skill descriptor files under roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when descriptor I/O or frontmatter parsing fails.
    pub fn build_from_roots(roots: &[PathBuf]) -> Result<Self, SkillVfsError> {
        let mut index = Self::default();
        let scanner = SkillScanner::new();

        for root in roots {
            if !root.exists() || !root.is_dir() {
                continue;
            }
            for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
                if !entry.file_type().is_file() {
                    continue;
                }
                let skill_doc = entry.into_path();
                if !is_skill_descriptor(skill_doc.as_path()) {
                    continue;
                }
                let Some(semantic_name) =
                    parse_semantic_name_from_skill_doc(skill_doc.as_path(), &scanner)?
                else {
                    continue;
                };
                let references_dir = skill_doc
                    .parent()
                    .map_or_else(PathBuf::new, |parent| parent.join("references"));
                index
                    .mounts_by_name
                    .entry(semantic_name.clone())
                    .or_default()
                    .push(SkillNamespaceMount {
                        semantic_name: semantic_name.clone(),
                        skill_doc,
                        references_dir,
                    });
                index.preload_references_for_semantic(&semantic_name);
            }
        }

        Ok(index)
    }

    /// Resolve all mounts by semantic namespace (case-insensitive).
    #[must_use]
    pub fn mounts_for(&self, semantic_name: &str) -> Option<&[SkillNamespaceMount]> {
        let key = semantic_name.trim().to_ascii_lowercase();
        self.mounts_by_name.get(&key).map(Vec::as_slice)
    }

    /// Returns total number of indexed semantic namespaces.
    #[must_use]
    pub fn namespace_count(&self) -> usize {
        self.mounts_by_name.len()
    }

    /// Resolve one concrete path from parsed semantic URI.
    #[must_use]
    pub fn path_for_uri(&self, uri: &super::WendaoResourceUri) -> Option<&PathBuf> {
        let key = semantic_resource_uri_key(uri.semantic_name(), uri.entity_name());
        self.paths_by_uri.get(key.as_str())
    }

    fn preload_references_for_semantic(&mut self, semantic_name: &str) {
        let Some(mounts) = self.mounts_by_name.get(semantic_name) else {
            return;
        };
        let references_roots = mounts
            .iter()
            .map(|mount| mount.references_dir.clone())
            .collect::<Vec<_>>();
        for references_dir in references_roots {
            preload_reference_dir(self, semantic_name, references_dir.as_path());
        }
    }
}

fn preload_reference_dir(
    index: &mut SkillNamespaceIndex,
    semantic_name: &str,
    references_dir: &Path,
) {
    if !references_dir.exists() || !references_dir.is_dir() {
        return;
    }
    for entry in WalkDir::new(references_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let absolute = entry.into_path();
        let Ok(relative) = absolute.strip_prefix(references_dir) else {
            continue;
        };
        let Some(relative_entity) = normalize_relative_entity_path(relative) else {
            continue;
        };
        let uri_key = semantic_resource_uri_key(semantic_name, relative_entity.as_str());
        if index.paths_by_uri.contains_key(uri_key.as_str()) {
            continue;
        }
        index.paths_by_uri.insert(uri_key, absolute);
    }
}

fn normalize_relative_entity_path(path: &Path) -> Option<String> {
    let rendered = path.to_string_lossy().replace('\\', "/");
    let trimmed = rendered.trim_matches('/');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn semantic_resource_uri_key(semantic_name: &str, entity_name: &str) -> String {
    format!(
        "wendao://skills/{}/references/{}",
        semantic_name.trim().to_ascii_lowercase(),
        entity_name.trim().trim_matches('/')
    )
}

fn is_skill_descriptor(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md") || name == "skill.md")
}

fn parse_semantic_name_from_skill_doc(
    path: &Path,
    scanner: &SkillScanner,
) -> Result<Option<String>, SkillVfsError> {
    if path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|name| name == "SKILL.md")
    {
        return parse_semantic_name_with_scanner(path, scanner);
    }

    let content =
        std::fs::read_to_string(path).map_err(|source| SkillVfsError::ReadSkillDescriptor {
            path: path.to_path_buf(),
            source,
        })?;
    parse_semantic_name_from_markdown(path, content.as_str())
}

fn parse_semantic_name_from_markdown(
    path: &Path,
    markdown: &str,
) -> Result<Option<String>, SkillVfsError> {
    let Some(value) = parse_frontmatter_from_markdown(markdown).map_err(|source| {
        SkillVfsError::ParseSkillFrontmatter {
            path: path.to_path_buf(),
            source,
        }
    })?
    else {
        return Ok(None);
    };
    let name = value
        .get("name")
        .and_then(serde_yaml::Value::as_str)
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(str::to_ascii_lowercase);
    Ok(name)
}

fn parse_semantic_name_with_scanner(
    path: &Path,
    scanner: &SkillScanner,
) -> Result<Option<String>, SkillVfsError> {
    let Some(skill_dir) = path.parent() else {
        return Ok(None);
    };
    let metadata =
        scanner
            .scan_skill(skill_dir, None)
            .map_err(|error| SkillVfsError::ScanSkillMetadata {
                path: path.to_path_buf(),
                reason: error.to_string(),
            })?;
    let semantic_name = metadata
        .as_ref()
        .map(|item| item.skill_name.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    Ok(semantic_name)
}
