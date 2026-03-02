use serde::{Deserialize, Serialize};

/// Represents an asset file discovered in a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AssetRecord {
    /// Name of the asset.
    pub asset_name: String,
    /// Title of the asset.
    pub title: String,
    /// Skill this asset belongs to.
    pub skill_name: String,
    /// Path to the asset file.
    pub file_path: String,
    /// Preview of the asset content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for asset discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the asset file.
    #[serde(default)]
    pub file_hash: String,
}

impl AssetRecord {
    /// Creates a new `AssetRecord` with required fields.
    #[must_use]
    pub fn new(asset_name: String, title: String, skill_name: String, file_path: String) -> Self {
        Self {
            asset_name,
            title,
            skill_name,
            file_path,
            content_preview: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}
