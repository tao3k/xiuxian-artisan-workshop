use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

use super::ReferencePath;

/// Parsed skill metadata from SKILL.md YAML frontmatter.
#[derive(Debug, Clone, Default, Deserialize, Serialize, SchemarsJsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct SkillMetadata {
    /// Unique name identifying this skill.
    #[serde(default)]
    pub skill_name: String,
    /// Semantic version string (e.g., "1.0.0").
    #[serde(default)]
    pub version: String,
    /// Human-readable description of the skill's purpose.
    #[serde(default)]
    pub description: String,
    /// Keywords used for semantic routing and skill selection.
    #[serde(default)]
    pub routing_keywords: Vec<String>,
    /// Authors who created or maintain this skill.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Intents this skill can handle (for intent-based routing).
    #[serde(default)]
    pub intents: Vec<String>,
    /// Paths to required reference files or skills.
    #[serde(default)]
    pub require_refs: Vec<ReferencePath>,
    /// Repository URL for the skill source code.
    #[serde(default)]
    pub repository: String,
    /// Permissions required by this skill (e.g., "filesystem:read", "network:http")
    /// Zero Trust: Empty permissions means NO access to any capabilities.
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl SkillMetadata {
    /// Creates a new empty `SkillMetadata` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `SkillMetadata` with the specified skill name.
    #[must_use]
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            skill_name: name.into(),
            ..Self::default()
        }
    }

    /// Returns `true` if the skill has routing keywords defined.
    #[must_use]
    pub fn has_routing_keywords(&self) -> bool {
        !self.routing_keywords.is_empty()
    }

    /// Returns a comma-separated summary of routing keywords.
    #[must_use]
    pub fn keywords_summary(&self) -> String {
        self.routing_keywords.join(", ")
    }
}
