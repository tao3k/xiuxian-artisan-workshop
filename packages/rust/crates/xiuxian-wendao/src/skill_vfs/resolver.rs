use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use include_dir::Dir;

use super::zhixing::{
    ZHIXING_EMBEDDED_CRATE_ID, embedded_resource_dir, embedded_semantic_reference_mounts,
};
use super::{SkillNamespaceIndex, SkillVfsError, WendaoResourceUri};

#[derive(Debug, Clone, PartialEq, Eq)]
struct EmbeddedSemanticMount {
    crate_id: String,
    references_dir: PathBuf,
}

/// Semantic resource resolver for `wendao://skills/.../references/...`.
#[derive(Debug, Clone, Default)]
pub struct SkillVfsResolver {
    index: SkillNamespaceIndex,
    mounts: HashMap<String, &'static Dir<'static>>,
    embedded_mounts_by_semantic: HashMap<String, Vec<EmbeddedSemanticMount>>,
    content_cache: Arc<DashMap<String, Arc<str>>>,
}

impl SkillVfsResolver {
    /// Build resolver by scanning one or more skill roots.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace indexing fails.
    pub fn from_roots(roots: &[PathBuf]) -> Result<Self, SkillVfsError> {
        Ok(Self {
            index: SkillNamespaceIndex::build_from_roots(roots)?,
            mounts: HashMap::new(),
            embedded_mounts_by_semantic: HashMap::new(),
            content_cache: Arc::new(DashMap::new()),
        })
    }

    /// Build resolver by scanning roots and enabling embedded resource mount.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace indexing fails.
    pub fn from_roots_with_embedded(roots: &[PathBuf]) -> Result<Self, SkillVfsError> {
        Self::from_roots(roots).map(Self::mount_embedded_dir)
    }

    /// Mount one embedded resource image and semantic reference map.
    ///
    /// `semantic_mounts` maps semantic name to one or more `references/` base
    /// directories that are relative to the mounted [`Dir`].
    #[must_use]
    pub fn mount(
        mut self,
        crate_id: &str,
        dir: &'static Dir<'static>,
        semantic_mounts: &HashMap<String, Vec<PathBuf>>,
    ) -> Self {
        let normalized_crate_id = crate_id.trim().to_ascii_lowercase();
        if normalized_crate_id.is_empty() {
            return self;
        }

        self.mounts.insert(normalized_crate_id.clone(), dir);
        for (semantic_name, references_dirs) in semantic_mounts {
            let semantic = semantic_name.trim().to_ascii_lowercase();
            if semantic.is_empty() {
                continue;
            }
            let entry = self
                .embedded_mounts_by_semantic
                .entry(semantic)
                .or_default();
            for references_dir in references_dirs {
                let mount = EmbeddedSemanticMount {
                    crate_id: normalized_crate_id.clone(),
                    references_dir: references_dir.clone(),
                };
                if !entry.iter().any(|existing| existing == &mount) {
                    entry.push(mount);
                }
            }
            entry.sort_by(|left, right| left.references_dir.cmp(&right.references_dir));
        }

        self
    }

    /// Enable embedded `include_dir` resource mount for semantic reads.
    #[must_use]
    pub fn mount_embedded_dir(mut self) -> Self {
        self = self.mount(
            ZHIXING_EMBEDDED_CRATE_ID,
            embedded_resource_dir(),
            embedded_semantic_reference_mounts(),
        );
        self
    }

    /// Access the underlying semantic namespace index.
    #[must_use]
    pub fn index(&self) -> &SkillNamespaceIndex {
        &self.index
    }

    /// Resolve one semantic URI to concrete file path.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI parsing fails, namespace is unknown,
    /// or no matching reference document exists.
    pub fn resolve_path(&self, uri: &str) -> Result<PathBuf, SkillVfsError> {
        let parsed = WendaoResourceUri::parse(uri)?;
        self.resolve_parsed_uri(&parsed)
    }

    /// Resolve one parsed URI to concrete file path.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when namespace is unknown or resource is missing.
    pub fn resolve_parsed_uri(&self, uri: &WendaoResourceUri) -> Result<PathBuf, SkillVfsError> {
        let Some(path) = self.index.path_for_uri(uri).cloned() else {
            let Some(_mounts) = self.index.mounts_for(uri.semantic_name()) else {
                return Err(SkillVfsError::UnknownSemanticSkill {
                    semantic_name: uri.semantic_name().to_string(),
                });
            };
            return Err(SkillVfsError::ResourceNotFound {
                semantic_name: uri.semantic_name().to_string(),
                entity_name: uri.entity_name().to_string(),
            });
        };

        Ok(path)
    }

    /// Resolve one semantic URI and return UTF-8 file content.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI resolution fails or content lookup fails.
    pub fn read_utf8(&self, uri: &str) -> Result<String, SkillVfsError> {
        self.read_semantic(uri)
            .map(|text| text.as_ref().to_string())
    }

    /// Resolve one semantic URI and return shared UTF-8 content.
    ///
    /// Runtime lookup is cache-backed and performs lazy disk reads on cache miss.
    ///
    /// Lookup order:
    /// 1. `content_cache` (shared interned payloads)
    /// 2. Local semantic reference path indexed by [`SkillNamespaceIndex`]
    /// 3. Embedded `include_dir` resources (when enabled)
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI resolution fails or content lookup fails.
    pub fn read_semantic(&self, uri: &str) -> Result<Arc<str>, SkillVfsError> {
        let parsed = WendaoResourceUri::parse(uri)?;
        let canonical_uri = parsed.canonical_uri();
        if let Some(cached) = self.content_cache.get(canonical_uri.as_str()) {
            return Ok(Arc::clone(cached.value()));
        }

        if let Some(shared) = self.read_local_semantic(&parsed, canonical_uri.as_str())? {
            return Ok(shared);
        }

        if let Some(shared) = self.read_mounted_semantic(&parsed, canonical_uri.as_str()) {
            return Ok(shared);
        }

        let semantic_known = self.index.mounts_for(parsed.semantic_name()).is_some()
            || self
                .embedded_mounts_by_semantic
                .contains_key(parsed.semantic_name());
        if !semantic_known {
            return Err(SkillVfsError::UnknownSemanticSkill {
                semantic_name: parsed.semantic_name().to_string(),
            });
        }
        Err(SkillVfsError::ResourceNotFound {
            semantic_name: parsed.semantic_name().to_string(),
            entity_name: parsed.entity_name().to_string(),
        })
    }

    /// Compatibility alias for existing callers that expect shared UTF-8 reads.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when URI resolution fails or content lookup fails.
    pub fn read_utf8_shared(&self, uri: &str) -> Result<Arc<str>, SkillVfsError> {
        self.read_semantic(uri)
    }

    fn read_mounted_semantic(
        &self,
        parsed: &WendaoResourceUri,
        canonical_uri: &str,
    ) -> Option<Arc<str>> {
        let mounts = self
            .embedded_mounts_by_semantic
            .get(parsed.semantic_name())?;
        let relative_entity = parsed.entity_relative_path();
        for mount in mounts {
            let Some(dir) = self.mounts.get(mount.crate_id.as_str()) else {
                continue;
            };
            let candidate = normalize_embedded_resource_path(
                mount.references_dir.join(relative_entity).as_path(),
            );
            let Some(file) = dir.get_file(candidate.as_str()) else {
                continue;
            };
            let Some(content) = file.contents_utf8() else {
                continue;
            };
            if content.trim().is_empty() {
                continue;
            }
            let shared = Arc::<str>::from(content);
            self.content_cache
                .insert(canonical_uri.to_string(), Arc::clone(&shared));
            return Some(shared);
        }
        None
    }

    fn read_local_semantic(
        &self,
        parsed: &WendaoResourceUri,
        canonical_uri: &str,
    ) -> Result<Option<Arc<str>>, SkillVfsError> {
        let Some(path) = self.index.path_for_uri(parsed).cloned() else {
            return Ok(None);
        };
        let content = std::fs::read_to_string(path.as_path()).map_err(|source| {
            SkillVfsError::ReadResource {
                path: path.clone(),
                source,
            }
        })?;
        let shared = Arc::<str>::from(content);
        self.content_cache
            .insert(canonical_uri.to_string(), Arc::clone(&shared));
        Ok(Some(shared))
    }
}

fn normalize_embedded_resource_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim()
        .trim_start_matches("./")
        .replace('\\', "/")
}
