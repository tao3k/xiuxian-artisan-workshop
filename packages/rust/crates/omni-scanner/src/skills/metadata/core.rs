use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

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

// =============================================================================
// Sniffer Rule - Declarative rules for skill activation
// =============================================================================

/// A single sniffer rule (typically from extensions/sniffer/rules.toml).
#[derive(Debug, Clone, Deserialize, Serialize, SchemarsJsonSchema, PartialEq, Eq)]
pub struct SnifferRule {
    /// Rule type: "`file_exists`" or "`file_pattern`"
    #[serde(rename = "type")]
    pub rule_type: String,
    /// Glob pattern or filename to match
    pub pattern: String,
}

impl SnifferRule {
    /// Creates a new `SnifferRule` with the given type and pattern.
    pub fn new(rule_type: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            rule_type: rule_type.into(),
            pattern: pattern.into(),
        }
    }
}

// =============================================================================
// Tool Annotations - MCP Protocol Safety Annotations
// =============================================================================

/// Safety and behavior annotations for tools (MCP Protocol compliant).
///
/// These annotations help the agent understand the safety implications
/// of using a tool, enabling smarter execution decisions.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, SchemarsJsonSchema)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct ToolAnnotations {
    /// Read-only operations that don't modify system state.
    #[serde(default)]
    pub read_only: bool,
    /// Operations that modify or delete data.
    #[serde(default)]
    pub destructive: bool,
    /// Operations that can be safely repeated without side effects.
    #[serde(default)]
    pub idempotent: bool,
    /// Operations that interact with external/open systems.
    #[serde(default)]
    pub open_world: bool,
}

impl ToolAnnotations {
    /// Creates a new `ToolAnnotations` with all defaults (safe defaults).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates annotations for a read-only tool.
    #[must_use]
    pub fn read_only() -> Self {
        Self {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: false,
        }
    }

    /// Creates annotations for a destructive tool.
    #[must_use]
    pub fn destructive() -> Self {
        Self {
            read_only: false,
            destructive: true,
            idempotent: false,
            open_world: false,
        }
    }

    /// Creates annotations for a network-accessible tool.
    #[must_use]
    pub fn open_world() -> Self {
        Self {
            read_only: false,
            destructive: false,
            idempotent: false,
            open_world: true,
        }
    }
}

// =============================================================================
// Decorator Arguments - Extracted from @skill_command decorator
// =============================================================================

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

// =============================================================================
// Tool Record
// =============================================================================

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

impl ToolRecord {
    /// Creates a new `ToolRecord` with required fields.
    #[must_use]
    pub fn new(
        tool_name: String,
        description: String,
        skill_name: String,
        file_path: String,
        function_name: String,
    ) -> Self {
        Self {
            tool_name,
            description,
            skill_name,
            file_path,
            function_name,
            execution_mode: String::new(),
            keywords: Vec::new(),
            intents: Vec::new(),
            file_hash: String::new(),
            input_schema: String::new(),
            docstring: String::new(),
            category: String::new(),
            annotations: ToolAnnotations::default(),
            parameters: Vec::new(),
            skill_tools_refers: Vec::new(),
            resource_uri: String::new(),
        }
    }

    /// Creates a fully populated `ToolRecord` by applying enrichment fields.
    #[must_use]
    pub fn with_enrichment(
        tool_name: String,
        description: String,
        skill_name: String,
        file_path: String,
        function_name: String,
        enrichment: ToolEnrichment,
    ) -> Self {
        Self {
            tool_name,
            description,
            skill_name,
            file_path,
            function_name,
            execution_mode: enrichment.execution_mode,
            keywords: enrichment.keywords,
            intents: enrichment.intents,
            file_hash: enrichment.file_hash,
            input_schema: enrichment.input_schema,
            docstring: enrichment.docstring,
            category: enrichment.category,
            annotations: enrichment.annotations,
            parameters: enrichment.parameters,
            skill_tools_refers: enrichment.skill_tools_refers,
            resource_uri: enrichment.resource_uri,
        }
    }
}

// =============================================================================
// Reference Path
// =============================================================================

/// A validated relative path to a reference document (md, pdf, txt, html, json, yaml, yml).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemarsJsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct ReferencePath(String);

impl ReferencePath {
    const VALID_EXTENSIONS: &[&str] = &["md", "pdf", "txt", "html", "json", "yaml", "yml"];

    /// Creates a new `ReferencePath` after validating the path format.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty, absolute, contains `..`,
    /// or has an unsupported file extension.
    pub fn new(path: impl Into<String>) -> Result<Self, String> {
        let path = path.into();
        if path.trim().is_empty() {
            return Err("Reference path cannot be empty".to_string());
        }
        if path.starts_with('/') {
            return Err(format!("Reference path must be relative: {path}"));
        }
        if path.contains("..") {
            return Err(format!("Reference path cannot contain '..': {path}"));
        }
        let ext = path.rsplit('.').next().unwrap_or("");
        if !ext.is_empty() && !Self::VALID_EXTENSIONS.contains(&ext) {
            return Err(format!("Invalid reference extension '{ext}'"));
        }
        Ok(Self(path))
    }

    /// Returns the reference path as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ReferencePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ReferencePath> for String {
    fn from(val: ReferencePath) -> Self {
        val.0
    }
}

impl TryFrom<String> for ReferencePath {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
