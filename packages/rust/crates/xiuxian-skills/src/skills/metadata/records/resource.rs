use serde::{Deserialize, Serialize};

/// Represents a discovered MCP Resource from @`skill_resource` decorated functions.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResourceRecord {
    /// Resource name (from decorator or function name).
    pub name: String,
    /// Human-readable description of the resource.
    pub description: String,
    /// Full resource URI (e.g., `<omni://skill/knowledge/graph_stats>`).
    pub resource_uri: String,
    /// MIME type of the resource content.
    pub mime_type: String,
    /// Name of the skill this resource belongs to.
    pub skill_name: String,
    /// File path where the resource provider is defined.
    pub file_path: String,
    /// Name of the function implementing this resource.
    pub function_name: String,
    /// Hash of the source file for change detection.
    pub file_hash: String,
}

impl ResourceRecord {
    /// Create a new `ResourceRecord`.
    #[must_use]
    pub fn new(
        name: String,
        description: String,
        resource_uri: String,
        skill_name: String,
        file_path: String,
        function_name: String,
        file_hash: String,
    ) -> Self {
        Self {
            name,
            description,
            resource_uri,
            mime_type: "application/json".to_string(),
            skill_name,
            file_path,
            function_name,
            file_hash,
        }
    }

    /// Override MIME type for non-JSON resources.
    #[must_use]
    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = mime_type;
        self
    }
}
