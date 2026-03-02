use serde::{Deserialize, Serialize};

/// Represents a data file discovered in a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DataRecord {
    /// Name of the data.
    pub data_name: String,
    /// Format of the data (e.g., "json", "csv").
    pub format: String,
    /// Skill this data belongs to.
    pub skill_name: String,
    /// Path to the data file.
    pub file_path: String,
    /// Field names in the data.
    pub fields: Vec<String>,
    /// Preview of the data content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for data discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the data file.
    #[serde(default)]
    pub file_hash: String,
}

impl DataRecord {
    /// Creates a new `DataRecord` with required fields.
    #[must_use]
    pub fn new(
        data_name: String,
        format: String,
        skill_name: String,
        file_path: String,
        fields: Vec<String>,
    ) -> Self {
        Self {
            data_name,
            format,
            skill_name,
            file_path,
            fields,
            content_preview: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}
