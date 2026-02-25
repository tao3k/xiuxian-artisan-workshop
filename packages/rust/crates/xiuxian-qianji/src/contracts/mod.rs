use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
