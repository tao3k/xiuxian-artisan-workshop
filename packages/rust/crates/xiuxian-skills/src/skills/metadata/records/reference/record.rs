use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

use super::serde_helpers::{de_opt_string_or_vec, de_string_or_vec, default_ref_doc_type};

/// Represents a reference document discovered in a skill's `references/` directory.
///
/// See `docs/reference/skill-data-hierarchy-and-references.md`: references are
/// subordinate to the skill; each may be tied to one or more skills/tools (e.g. graph docs).
/// `for_skills` and `for_tools` are lists so one reference can apply to multiple skills or tools.
#[derive(Debug, Clone, Deserialize, Serialize, SchemarsJsonSchema, PartialEq, Eq)]
pub struct ReferenceRecord {
    /// Name of the reference (e.g. filename stem).
    pub ref_name: String,
    /// Title of the reference document (from frontmatter or first heading).
    pub title: String,
    /// Primary skill (first of `for_skills` or parent path); kept for backward compatibility.
    pub skill_name: String,
    /// Path to the reference file (relative to repo or absolute).
    pub file_path: String,
    /// List of skills this reference applies to; derived from `for_tools` (skill part of each `skill.tool`) or from path when `for_tools` is absent.
    #[serde(default, deserialize_with = "de_string_or_vec")]
    pub for_skills: Vec<String>,
    /// If set, list of full tool names this reference is for (e.g. `["git.smart_commit", "researcher.run_research_graph"]`).
    #[serde(default, deserialize_with = "de_opt_string_or_vec", alias = "for_tool")]
    pub for_tools: Option<Vec<String>>,
    /// Document type: `"reference"` for references/*.md; `"comprehensive"` reserved for SKILL.md.
    #[serde(default = "default_ref_doc_type")]
    pub doc_type: String,
    /// Preview of the content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for reference discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Section headings in the document.
    #[serde(default)]
    pub sections: Vec<String>,
    /// Hash of the reference file.
    #[serde(default)]
    pub file_hash: String,
}

impl ReferenceRecord {
    /// Creates a new `ReferenceRecord` with required fields.
    #[must_use]
    pub fn new(ref_name: String, title: String, skill_name: String, file_path: String) -> Self {
        let for_skills = if skill_name.is_empty() {
            Vec::new()
        } else {
            vec![skill_name.clone()]
        };

        Self {
            ref_name,
            title,
            skill_name,
            file_path,
            for_skills,
            for_tools: None,
            doc_type: "reference".to_string(),
            content_preview: String::new(),
            keywords: Vec::new(),
            sections: Vec::new(),
            file_hash: String::new(),
        }
    }

    /// Builder: set optional list of tools this reference is for.
    #[must_use]
    pub fn with_for_tools(mut self, for_tools: Option<Vec<String>>) -> Self {
        self.for_tools = for_tools;
        self
    }

    /// Returns true if this reference applies to the given full tool name.
    #[must_use]
    pub fn applies_to_tool(&self, full_tool_name: &str) -> bool {
        self.for_tools
            .as_ref()
            .is_some_and(|tools| tools.iter().any(|tool| tool.as_str() == full_tool_name))
    }
}
