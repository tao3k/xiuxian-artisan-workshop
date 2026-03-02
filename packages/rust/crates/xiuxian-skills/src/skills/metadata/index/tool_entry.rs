use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

/// A tool entry in the skill index.
#[derive(Debug, Clone, Serialize, Deserialize, SchemarsJsonSchema, PartialEq, Eq)]
pub struct IndexToolEntry {
    /// Name of the tool.
    pub name: String,
    /// Description of what the tool does.
    pub description: String,
    /// Category for organizing tools (e.g., "read", "write", "query").
    #[serde(default)]
    pub category: String,
    /// JSON schema for tool input validation (MCP protocol format).
    #[serde(default)]
    pub input_schema: String,
    /// Hash of the source file for incremental sync.
    #[serde(default)]
    pub file_hash: String,
}
