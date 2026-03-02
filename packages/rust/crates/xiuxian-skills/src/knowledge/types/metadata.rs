use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

use super::KnowledgeCategory;

/// Metadata extracted from knowledge document frontmatter.
#[derive(Debug, Clone, Default, Serialize, Deserialize, SchemarsJsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct KnowledgeMetadata {
    /// Document title (optional, extracted from first H1 if not present)
    #[serde(default)]
    pub title: Option<String>,
    /// Human-readable description of the document
    #[serde(default)]
    pub description: Option<String>,
    /// Category for organization and filtering
    #[serde(default)]
    pub category: Option<KnowledgeCategory>,
    /// Tags for semantic search and discovery
    #[serde(default)]
    pub tags: Vec<String>,
    /// Authors who created or maintain this document
    #[serde(default)]
    pub authors: Vec<String>,
    /// Source file or URL
    #[serde(default)]
    pub source: Option<String>,
    /// Version for tracking document changes
    #[serde(default)]
    pub version: Option<String>,
}

impl KnowledgeMetadata {
    /// Create a new empty `KnowledgeMetadata`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a category.
    #[must_use]
    pub fn with_category(mut self, category: KnowledgeCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags.
    #[must_use]
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags
            .extend(tags.into_iter().map(std::convert::Into::into));
        self
    }
}
