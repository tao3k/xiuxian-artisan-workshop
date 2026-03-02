use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

/// Documentation availability status for a skill.
#[derive(Debug, Clone, Serialize, Deserialize, SchemarsJsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocsAvailable {
    /// Whether SKILL.md exists.
    #[serde(default)]
    pub skill_md: bool,
    /// Whether README.md exists.
    #[serde(default)]
    pub readme: bool,
    /// Whether tests exist.
    #[serde(default)]
    pub tests: bool,
}

impl Default for DocsAvailable {
    fn default() -> Self {
        Self {
            skill_md: true,
            readme: false,
            tests: false,
        }
    }
}
