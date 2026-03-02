use schemars::JsonSchema as SchemarsJsonSchema;
use serde::{Deserialize, Serialize};

/// Safety and behavior annotations for tools (MCP Protocol compliant).
///
/// These annotations help the agent understand the safety implications
/// of using a tool, enabling smarter execution decisions.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, SchemarsJsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolBehaviorAnnotations {
    /// Operations that can be safely repeated without side effects.
    #[serde(default)]
    pub idempotent: bool,
    /// Operations that interact with external/open systems.
    #[serde(default)]
    pub open_world: bool,
}

/// Safety and behavior annotations for tools (MCP Protocol compliant).
///
/// The behavior flags are flattened to keep the external JSON shape stable
/// while avoiding an excessive-bool anti-pattern in a single struct.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, SchemarsJsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    /// Read-only operations that don't modify system state.
    #[serde(default)]
    pub read_only: bool,
    /// Operations that modify or delete data.
    #[serde(default)]
    pub destructive: bool,
    /// Behavioral flags flattened into the top-level annotation object.
    #[serde(flatten)]
    pub behavior: ToolBehaviorAnnotations,
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
            behavior: ToolBehaviorAnnotations {
                idempotent: true,
                open_world: false,
            },
        }
    }

    /// Creates annotations for a destructive tool.
    #[must_use]
    pub fn destructive() -> Self {
        Self {
            read_only: false,
            destructive: true,
            behavior: ToolBehaviorAnnotations {
                idempotent: false,
                open_world: false,
            },
        }
    }

    /// Creates annotations for a network-accessible tool.
    #[must_use]
    pub fn open_world() -> Self {
        Self {
            read_only: false,
            destructive: false,
            behavior: ToolBehaviorAnnotations {
                idempotent: false,
                open_world: true,
            },
        }
    }

    /// Returns whether the tool is idempotent.
    #[must_use]
    pub const fn is_idempotent(&self) -> bool {
        self.behavior.idempotent
    }

    /// Sets idempotent flag.
    pub fn set_idempotent(&mut self, idempotent: bool) {
        self.behavior.idempotent = idempotent;
    }

    /// Returns whether the tool is open-world.
    #[must_use]
    pub const fn is_open_world(&self) -> bool {
        self.behavior.open_world
    }

    /// Sets open-world flag.
    pub fn set_open_world(&mut self, open_world: bool) {
        self.behavior.open_world = open_world;
    }
}
