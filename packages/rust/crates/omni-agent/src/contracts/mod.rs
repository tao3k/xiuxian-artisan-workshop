mod discover;
mod graph_plan;
mod memory_gate;
mod omega;
mod route_trace;

pub use discover::{DiscoverConfidence, DiscoverMatch};
pub use graph_plan::{GraphExecutionPlan, GraphPlanStep, GraphPlanStepKind, GraphWorkflowMode};
pub use memory_gate::{MemoryGateDecision, MemoryGateVerdict};
pub use omega::{
    OmegaDecision, OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass,
};
pub use route_trace::{RouteTrace, RouteTraceGraphStep, RouteTraceInjection};

/// Explicit REPL routing mode for workflow bridge contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkflowBridgeMode {
    /// Direct graph execution.
    Graph,
    /// High-level omega reasoning.
    Omega,
}

impl WorkflowBridgeMode {
    /// Returns the string representation of the mode.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Graph => "graph",
            Self::Omega => "omega",
        }
    }
}
