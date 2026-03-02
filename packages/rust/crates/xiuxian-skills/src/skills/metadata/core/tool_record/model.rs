use serde::{Deserialize, Serialize};

use super::super::ToolAnnotations;

/// Represents a discovered tool/function within a skill.
///
/// This struct is enriched with metadata extracted from:
/// - AST parsing of decorator kwargs
/// - Function signature analysis
/// - Docstring parsing
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolRecord {
    /// Name of the tool function.
    pub tool_name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// Name of the skill this tool belongs to.
    pub skill_name: String,
    /// File path where the tool is defined.
    pub file_path: String,
    /// Name of the function implementing this tool.
    pub function_name: String,
    /// Execution mode (e.g., "sync", "async", "script").
    pub execution_mode: String,
    /// Keywords for tool discovery and routing.
    pub keywords: Vec<String>,
    /// Intents this tool can fulfill (inherited from skill).
    #[serde(default)]
    pub intents: Vec<String>,
    /// Hash of the source file for change detection.
    pub file_hash: String,
    /// JSON schema for tool input validation.
    #[serde(default)]
    pub input_schema: String,
    /// Documentation string from the function docstring.
    #[serde(default)]
    pub docstring: String,
    /// Category inferred from decorator or function signature.
    #[serde(default)]
    pub category: String,
    /// MCP protocol safety annotations.
    #[serde(default)]
    pub annotations: ToolAnnotations,
    /// Parameter names inferred from function signature.
    #[serde(default)]
    pub parameters: Vec<String>,
    /// Full tool names (skill.tool) this skill tool refers to for docs (`SkillToolsRefers`).
    #[serde(default)]
    pub skill_tools_refers: Vec<String>,
    /// MCP Resource URI.  Non-empty means this tool is also an MCP Resource.
    #[serde(default)]
    pub resource_uri: String,
}

/// Enrichment payload applied to a base `ToolRecord`.
///
/// This groups optional/computed metadata so tool construction remains
/// explicit without relying on a long argument list.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ToolEnrichment {
    /// Execution mode (e.g., "sync", "async", "script").
    pub execution_mode: String,
    /// Keywords for tool discovery and routing.
    pub keywords: Vec<String>,
    /// Intents this tool can fulfill (inherited from skill).
    #[serde(default)]
    pub intents: Vec<String>,
    /// Hash of the source file for change detection.
    pub file_hash: String,
    /// Documentation string from the function docstring.
    #[serde(default)]
    pub docstring: String,
    /// Category inferred from decorator or function signature.
    #[serde(default)]
    pub category: String,
    /// MCP protocol safety annotations.
    #[serde(default)]
    pub annotations: ToolAnnotations,
    /// Parameter names inferred from function signature.
    #[serde(default)]
    pub parameters: Vec<String>,
    /// JSON schema for tool input validation.
    #[serde(default)]
    pub input_schema: String,
    /// Full tool names (skill.tool) this skill tool refers to for docs (`SkillToolsRefers`).
    #[serde(default)]
    pub skill_tools_refers: Vec<String>,
    /// MCP Resource URI. Non-empty means this tool is also an MCP Resource.
    #[serde(default)]
    pub resource_uri: String,
}
