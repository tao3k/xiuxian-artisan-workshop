use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

/// Knowledge categories for organizing documents.
///
/// Categories are used for filtering and routing knowledge queries.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, SchemarsJsonSchema, PartialEq, Eq, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeCategory {
    /// Architecture and design patterns
    Architecture,
    /// Debugging guides and solutions
    Debugging,
    /// Error handling patterns
    Error,
    /// General notes and informal documentation
    Note,
    /// Best practices and patterns
    Pattern,
    /// Reference documentation
    Reference,
    /// How-to guides and techniques
    Technique,
    /// Workflow documentation
    Workflow,
    /// Solution-oriented documentation
    Solution,
    /// Uncategorized
    #[serde(other)]
    #[default]
    Unknown,
}

impl std::str::FromStr for KnowledgeCategory {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "architecture" | "arch" => Ok(Self::Architecture),
            "debugging" | "debug" => Ok(Self::Debugging),
            "error" | "err" => Ok(Self::Error),
            "note" | "notes" => Ok(Self::Note),
            "pattern" | "patterns" => Ok(Self::Pattern),
            "reference" | "ref" => Ok(Self::Reference),
            "technique" | "techniques" => Ok(Self::Technique),
            "workflow" | "workflows" => Ok(Self::Workflow),
            "solution" | "solutions" => Ok(Self::Solution),
            _ => Ok(Self::Unknown),
        }
    }
}

impl std::fmt::Display for KnowledgeCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Architecture => write!(formatter, "architecture"),
            Self::Debugging => write!(formatter, "debugging"),
            Self::Error => write!(formatter, "error"),
            Self::Note => write!(formatter, "note"),
            Self::Pattern => write!(formatter, "pattern"),
            Self::Reference => write!(formatter, "reference"),
            Self::Technique => write!(formatter, "technique"),
            Self::Workflow => write!(formatter, "workflow"),
            Self::Solution => write!(formatter, "solution"),
            Self::Unknown => write!(formatter, "unknown"),
        }
    }
}
