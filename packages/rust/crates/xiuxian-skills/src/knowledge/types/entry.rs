use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

use super::KnowledgeCategory;

/// Represents a discovered knowledge document.
///
/// Contains metadata extracted from YAML frontmatter along with
/// file path and content information.
#[derive(Debug, Clone, Serialize, Deserialize, SchemarsJsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntry {
    /// Unique identifier for this entry
    pub id: String,
    /// Relative path to the document
    pub file_path: String,
    /// Document title
    pub title: String,
    /// Document description
    #[serde(default)]
    pub description: String,
    /// Category for organization
    #[serde(default)]
    pub category: KnowledgeCategory,
    /// Tags for semantic search
    #[serde(default)]
    pub tags: Vec<String>,
    /// Authors
    #[serde(default)]
    pub authors: Vec<String>,
    /// Source URL or file path
    #[serde(default)]
    pub source: Option<String>,
    /// Version identifier
    #[serde(default)]
    pub version: String,
    /// SHA256 hash of file content for change detection
    pub file_hash: String,
    /// Content preview (first N characters)
    #[serde(default)]
    pub content_preview: String,
}

impl KnowledgeEntry {
    /// Create a new `KnowledgeEntry`.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        file_path: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            file_path: file_path.into(),
            title: title.into(),
            description: String::new(),
            category: KnowledgeCategory::Unknown,
            tags: Vec::new(),
            authors: Vec::new(),
            source: None,
            version: String::new(),
            file_hash: String::new(),
            content_preview: String::new(),
        }
    }

    /// Get the category as a string.
    #[must_use]
    pub fn category_str(&self) -> &str {
        match self.category {
            KnowledgeCategory::Architecture => "architecture",
            KnowledgeCategory::Debugging => "debugging",
            KnowledgeCategory::Error => "error",
            KnowledgeCategory::Note => "note",
            KnowledgeCategory::Pattern => "pattern",
            KnowledgeCategory::Reference => "reference",
            KnowledgeCategory::Technique => "technique",
            KnowledgeCategory::Workflow => "workflow",
            KnowledgeCategory::Solution => "solution",
            KnowledgeCategory::Unknown => "unknown",
        }
    }
}

impl Default for KnowledgeEntry {
    fn default() -> Self {
        Self::new("", "", "")
    }
}
