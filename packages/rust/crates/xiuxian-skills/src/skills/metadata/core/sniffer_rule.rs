use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

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
