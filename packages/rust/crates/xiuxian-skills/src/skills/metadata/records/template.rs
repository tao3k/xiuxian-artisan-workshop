use serde::{Deserialize, Serialize};

/// Represents a discovered template file within a skill.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TemplateRecord {
    /// Name of the template.
    pub template_name: String,
    /// Description of the template's purpose.
    pub description: String,
    /// Skill this template belongs to.
    pub skill_name: String,
    /// Path to the template file.
    pub file_path: String,
    /// Variable names used in the template.
    pub variables: Vec<String>,
    /// Preview of the template content.
    #[serde(default)]
    pub content_preview: String,
    /// Keywords for template discovery.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Hash of the template file.
    #[serde(default)]
    pub file_hash: String,
}

impl TemplateRecord {
    /// Creates a new `TemplateRecord` with required fields.
    #[must_use]
    pub fn new(
        template_name: String,
        description: String,
        skill_name: String,
        file_path: String,
        variables: Vec<String>,
    ) -> Self {
        Self {
            template_name,
            description,
            skill_name,
            file_path,
            variables,
            content_preview: String::new(),
            keywords: Vec::new(),
            file_hash: String::new(),
        }
    }
}
