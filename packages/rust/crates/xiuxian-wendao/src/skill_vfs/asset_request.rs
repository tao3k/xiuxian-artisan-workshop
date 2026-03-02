use std::collections::HashMap;
use std::path::{Component, Path};
use std::sync::{Arc, OnceLock, RwLock};

use super::SkillVfsError;
use super::zhixing::embedded_resource_text_from_wendao_uri;

static STRIPPED_BODY_CACHE: OnceLock<RwLock<HashMap<String, Arc<str>>>> = OnceLock::new();

/// Strongly-typed handle for building semantic Wendao asset requests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WendaoAssetHandle;

impl WendaoAssetHandle {
    /// Builds one skill reference request:
    /// `wendao://skills/<semantic_name>/references/<relative_reference_path>`.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError`] when semantic name or relative path is invalid.
    pub fn skill_reference_asset(
        semantic_name: &str,
        relative_reference_path: &str,
    ) -> Result<AssetRequest, SkillVfsError> {
        let normalized_semantic = normalize_package_id(semantic_name).ok_or_else(|| {
            SkillVfsError::InvalidUri(format!(
                "wendao://skills/{semantic_name}/references/{relative_reference_path}"
            ))
        })?;
        let normalized_reference = normalize_relative_asset_path(relative_reference_path)?;
        Ok(AssetRequest::new(format!(
            "wendao://skills/{normalized_semantic}/references/{normalized_reference}"
        )))
    }
}

/// Chainable, typed Wendao URI request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRequest {
    uri: String,
}

impl AssetRequest {
    /// Creates one request from a full semantic URI.
    #[must_use]
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    /// Returns full semantic URI.
    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Reads asset text using the caller-provided resolver callback.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when the callback
    /// returns `None` for this URI.
    pub fn read_utf8_with<F>(&self, resolver: F) -> Result<String, SkillVfsError>
    where
        F: Fn(&str) -> Option<String>,
    {
        resolver(self.uri())
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| SkillVfsError::EmbeddedAssetNotFound {
                uri: self.uri.clone(),
            })
    }

    /// Reads UTF-8 asset text through Wendao's built-in embedded resolver.
    ///
    /// This method currently supports embedded skill assets that are resolvable
    /// by Wendao internal registries.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when no embedded asset
    /// exists for this URI.
    pub fn read_utf8(&self) -> Result<String, SkillVfsError> {
        embedded_resource_text_from_wendao_uri(self.uri())
            .map(str::to_string)
            .ok_or_else(|| SkillVfsError::EmbeddedAssetNotFound {
                uri: self.uri.clone(),
            })
    }

    /// Reads UTF-8 asset text through Wendao's built-in embedded resolver and
    /// returns a shared immutable string.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when no embedded asset
    /// exists for this URI.
    pub fn read_utf8_shared(&self) -> Result<Arc<str>, SkillVfsError> {
        embedded_resource_text_from_wendao_uri(self.uri())
            .map(Arc::<str>::from)
            .ok_or_else(|| SkillVfsError::EmbeddedAssetNotFound {
                uri: self.uri.clone(),
            })
    }

    /// Reads asset text and trims outer whitespace.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when resolver lookup
    /// fails or yields empty content.
    pub fn read_stripped_body_with<F>(&self, resolver: F) -> Result<String, SkillVfsError>
    where
        F: Fn(&str) -> Option<String>,
    {
        self.read_stripped_body_with_shared(resolver)
            .map(|text| text.as_ref().to_string())
    }

    /// Reads and strips one asset through Wendao's built-in embedded resolver.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when no embedded asset
    /// exists for this URI.
    pub fn read_stripped_body(&self) -> Result<String, SkillVfsError> {
        self.read_stripped_body_shared()
            .map(|text| text.as_ref().to_string())
    }

    /// Reads and strips one asset using the caller-provided resolver callback
    /// and returns a shared immutable string.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when resolver lookup
    /// fails or yields empty content.
    pub fn read_stripped_body_with_shared<F>(&self, resolver: F) -> Result<Arc<str>, SkillVfsError>
    where
        F: Fn(&str) -> Option<String>,
    {
        self.read_utf8_with(resolver).map(|text| {
            let stripped = text.trim();
            Arc::<str>::from(stripped)
        })
    }

    /// Reads and strips one asset through Wendao's built-in embedded resolver
    /// with process-level `Arc<str>` cache.
    ///
    /// # Errors
    ///
    /// Returns [`SkillVfsError::EmbeddedAssetNotFound`] when no embedded asset
    /// exists for this URI.
    pub fn read_stripped_body_shared(&self) -> Result<Arc<str>, SkillVfsError> {
        let cache = stripped_body_cache();
        if let Some(hit) = cache
            .read()
            .ok()
            .and_then(|entries| entries.get(self.uri()).cloned())
        {
            return Ok(hit);
        }

        let stripped = embedded_resource_text_from_wendao_uri(self.uri())
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(Arc::<str>::from)
            .ok_or_else(|| SkillVfsError::EmbeddedAssetNotFound {
                uri: self.uri.clone(),
            })?;

        if let Ok(mut entries) = cache.write() {
            entries.insert(self.uri.clone(), Arc::clone(&stripped));
        }
        Ok(stripped)
    }
}

fn stripped_body_cache() -> &'static RwLock<HashMap<String, Arc<str>>> {
    STRIPPED_BODY_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn normalize_package_id(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase().replace('_', "-");
    if normalized.is_empty() {
        return None;
    }
    if normalized
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        Some(normalized)
    } else {
        None
    }
}

fn normalize_relative_asset_path(raw: &str) -> Result<String, SkillVfsError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(SkillVfsError::InvalidRelativeAssetPath {
            path: raw.to_string(),
        });
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(SkillVfsError::InvalidRelativeAssetPath {
            path: raw.to_string(),
        });
    }
    let mut normalized = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                let rendered = value.to_string_lossy().trim().to_string();
                if rendered.is_empty() {
                    return Err(SkillVfsError::InvalidRelativeAssetPath {
                        path: raw.to_string(),
                    });
                }
                normalized.push(rendered);
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(SkillVfsError::InvalidRelativeAssetPath {
                    path: raw.to_string(),
                });
            }
        }
    }
    if normalized.is_empty() {
        return Err(SkillVfsError::InvalidRelativeAssetPath {
            path: raw.to_string(),
        });
    }
    Ok(normalized.join("/"))
}
