use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

use crate::skills::metadata::{ReferencePath, ReferenceRecord, SnifferRule};

use super::{DocsAvailable, IndexToolEntry};

/// Represents a skill entry in the skill index (skills.json).
#[derive(Debug, Clone, Serialize, Deserialize, SchemarsJsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillIndexEntry {
    /// Name of the skill.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Semantic version.
    pub version: String,
    /// Relative path to the skill directory.
    pub path: String,
    /// List of tools provided by this skill.
    pub tools: Vec<IndexToolEntry>,
    /// Keywords for semantic routing.
    pub routing_keywords: Vec<String>,
    /// Intents this skill handles.
    pub intents: Vec<String>,
    /// Authors of the skill.
    pub authors: Vec<String>,
    /// Documentation availability status.
    #[serde(default)]
    pub docs_available: DocsAvailable,
    /// Open source compliance status.
    #[serde(default)]
    pub oss_compliant: Vec<String>,
    /// Compliance check details.
    #[serde(default)]
    pub compliance_details: Vec<String>,
    /// Required reference paths.
    #[serde(default)]
    pub require_refs: Vec<ReferencePath>,
    /// Sniffer rules for skill activation (declarative).
    #[serde(default)]
    pub sniffing_rules: Vec<SnifferRule>,
    /// Permissions declared by this skill (Zero Trust: empty = no access).
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Reference docs from `references/*.md` (`metadata.for_tools` per doc).
    #[serde(default)]
    pub references: Vec<ReferenceRecord>,
}

impl SkillIndexEntry {
    /// Creates a new `SkillIndexEntry` with required fields.
    #[must_use]
    pub fn new(name: String, description: String, version: String, path: String) -> Self {
        Self {
            name,
            description,
            version,
            path,
            tools: Vec::new(),
            routing_keywords: Vec::new(),
            intents: Vec::new(),
            authors: vec!["omni-dev-fusion".to_string()],
            docs_available: DocsAvailable::default(),
            oss_compliant: Vec::new(),
            compliance_details: Vec::new(),
            require_refs: Vec::new(),
            sniffing_rules: Vec::new(),
            permissions: Vec::new(),
            references: Vec::new(),
        }
    }

    /// Adds a tool to this skill entry.
    pub fn add_tool(&mut self, tool: IndexToolEntry) {
        self.tools.push(tool);
    }

    /// Returns `true` if the skill has at least one tool.
    #[must_use]
    pub fn has_tools(&self) -> bool {
        !self.tools.is_empty()
    }
}
