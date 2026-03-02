use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::consensus::ConsensusPolicy;

/// Represents the execution status of a single mechanism node in the Qianji Box.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// Initial state before any execution attempt.
    Idle,
    /// Waiting in the scheduling queue.
    Queued,
    /// Currently performing logic.
    Executing,
    /// Waiting for multi-agent consensus agreement.
    ConsensusPending,
    /// Under adversarial audit (Synapse-Audit).
    Calibrating,
    /// Successfully finished execution.
    Completed,
    /// Terminal failure with an error message.
    Failed(String),
}

/// Control instructions emitted by nodes to manipulate the workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowInstruction {
    /// Continue to next topological layer normally.
    Continue,
    /// Select a specific outgoing edge by its label (Probabilistic/Conditional).
    SelectBranch(String),
    /// Reset specific nodes to 'Idle' and restart their execution (Calibration Loop).
    RetryNodes(Vec<String>),
    /// Suspend workflow execution, save checkpoint, and yield control back to the caller.
    Suspend(String),
    /// Terminate the entire workflow immediately with a fatal error.
    Abort(String),
}

/// Structured output from a Qianji mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QianjiOutput {
    /// The resulting data to be merged into the shared context.
    pub data: serde_json::Value,
    /// The routing/flow instruction for the scheduler.
    pub instruction: FlowInstruction,
}

/// Execution mode for per-node Qianhuan annotation bindings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeQianhuanExecutionMode {
    /// Use an isolated ephemeral injection window for each node execution.
    ///
    /// This is the default mode for multi-persona adversarial loops to avoid
    /// context contamination across nodes.
    #[default]
    Isolated,
    /// Reuse and append to a continuous history window via a context key.
    ///
    /// Use this mode only for same-persona multi-step tool execution chains.
    Appended,
}

impl NodeQianhuanExecutionMode {
    /// Returns the stable string representation used in telemetry payloads.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Isolated => "isolated",
            Self::Appended => "appended",
        }
    }
}

/// Qianhuan binding metadata attached to a node.
///
/// This formalizes Phase E of the Qianji-Qianhuan interface in TOML:
/// `[[nodes]] ... [nodes.qianhuan]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct NodeQianhuanBinding {
    /// Persona profile identifier resolved by `PersonaRegistry`.
    pub persona_id: Option<String>,
    /// Logical template target consumed by manifestation/runtime layers.
    pub template_target: Option<String>,
    /// Execution-mode selector for context window behavior.
    #[serde(default)]
    pub execution_mode: NodeQianhuanExecutionMode,
    /// Whitelisted context keys that can be marshaled into this node's
    /// annotation narrative blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_keys: Vec<String>,
    /// Context key used for appended mode history persistence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_key: Option<String>,
    /// Output context key that stores the generated annotation snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_key: Option<String>,
}

/// LLM tenant binding metadata attached to a node.
///
/// This enables node-scoped provider/model selection in TOML:
/// `[[nodes]] ... [nodes.llm]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct NodeLlmBinding {
    /// Optional backend/provider identifier (for example `openai`, `litellm_rs`).
    pub provider: Option<String>,
    /// Optional model override for this node.
    pub model: Option<String>,
    /// Optional OpenAI-compatible base URL override for this node.
    pub base_url: Option<String>,
    /// Optional environment variable name containing API key for this node.
    pub api_key_env: Option<String>,
}

/// Definition of a node in the declarative manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Unique identifier for the node.
    pub id: String,
    /// Type of task (e.g., knowledge, annotation).
    pub task_type: String,
    /// Priority weight for scheduling.
    pub weight: f32,
    /// Task-specific parameters.
    pub params: serde_json::Value,
    /// Optional node-level Qianhuan binding metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qianhuan: Option<NodeQianhuanBinding>,
    /// Optional node-level LLM tenant binding metadata.
    ///
    /// Backward compatibility:
    /// - preferred table: `[nodes.llm]`
    /// - legacy alias: `[nodes.llm_config]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "llm_config")]
    pub llm: Option<NodeLlmBinding>,
    /// Optional consensus policy for distributed voting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consensus: Option<ConsensusPolicy>,
}

/// Definition of an edge between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefinition {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Optional label for branch selection.
    pub label: Option<String>,
    /// Transition weight.
    pub weight: f32,
}

/// Declarative manifest for a Qianji workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QianjiManifest {
    /// Name of the pipeline.
    pub name: String,
    /// All node definitions.
    pub nodes: Vec<NodeDefinition>,
    /// All edge definitions.
    #[serde(default)]
    pub edges: Vec<EdgeDefinition>,
}

/// The Holy Trait for every Qianji Mechanism.
///
/// Every implementation serves as an interlocking gear in the Thousand Mechanism box.
#[async_trait]
pub trait QianjiMechanism: Send + Sync {
    /// Executes the core logic of the mechanism with access to the shared context.
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String>;
    /// Returns the scheduling weight/priority.
    fn weight(&self) -> f32; // For probabilistic routing
}
