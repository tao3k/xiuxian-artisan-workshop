use serde::{Deserialize, Serialize};

/// Arguments extracted from @`skill_command` decorator kwargs.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct DecoratorArgs {
    /// Explicit tool name from decorator (overrides function name).
    #[serde(default)]
    pub name: Option<String>,
    /// Human-readable description of what the tool does.
    #[serde(default)]
    pub description: Option<String>,
    /// Category for organizing tools (e.g., "read", "write", "query").
    #[serde(default)]
    pub category: Option<String>,
    /// Whether this tool modifies external state.
    #[serde(default)]
    pub destructive: Option<bool>,
    /// Whether this tool only reads data.
    #[serde(default)]
    pub read_only: Option<bool>,
    /// MCP Resource URI.  When set, this command is also exposed as a
    /// read-only MCP Resource at the given URI (e.g. `omni://skill/git/status`).
    #[serde(default)]
    pub resource_uri: Option<String>,
}
