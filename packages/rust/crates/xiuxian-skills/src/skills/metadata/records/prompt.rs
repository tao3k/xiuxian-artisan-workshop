use serde::{Deserialize, Serialize};

/// Represents a discovered MCP Prompt from @prompt decorated functions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PromptRecord {
    /// Prompt name (from decorator or function name).
    pub name: String,
    /// Human-readable description of the prompt.
    pub description: String,
    /// Name of the skill this prompt belongs to.
    pub skill_name: String,
    /// File path where the prompt is defined.
    pub file_path: String,
    /// Name of the function implementing this prompt.
    pub function_name: String,
    /// Hash of the source file for change detection.
    pub file_hash: String,
    /// Parameter names for the prompt template.
    #[serde(default)]
    pub parameters: Vec<String>,
}

impl PromptRecord {
    /// Create a new `PromptRecord`.
    #[must_use]
    pub fn new(
        name: String,
        description: String,
        skill_name: String,
        file_path: String,
        function_name: String,
        file_hash: String,
        parameters: Vec<String>,
    ) -> Self {
        Self {
            name,
            description,
            skill_name,
            file_path,
            function_name,
            file_hash,
            parameters,
        }
    }
}
