//! Core graph engine based on petgraph.

use crate::consensus::ConsensusPolicy;
use crate::contracts::{NodeStatus, QianjiMechanism};
use petgraph::Directed;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use std::sync::Arc;

/// Compiler for declarative manifests.
pub mod compiler;

/// Optional execution affinity used for role-aware swarm scheduling.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeExecutionAffinity {
    /// Execute this node only on the matching agent id when set.
    pub agent_id: Option<String>,
    /// Execute this node only on the matching role class when set.
    pub role_class: Option<String>,
}

/// Represents a single thought mechanism node in the execution graph.
#[derive(Clone)]
pub struct QianjiNode {
    /// Unique ID of the node.
    pub id: String,
    /// Current execution status.
    pub status: NodeStatus,
    /// The logic to be executed.
    pub mechanism: Arc<dyn QianjiMechanism>,
    /// Optional consensus policy.
    pub consensus: Option<ConsensusPolicy>,
    /// Optional execution affinity for distributed swarm routing.
    pub execution_affinity: NodeExecutionAffinity,
}

/// Represents an edge between nodes with optional label and weight.
#[derive(Debug, Clone)]
pub struct QianjiEdge {
    /// Label for branch selection.
    pub label: Option<String>,
    /// Probability/Priority weight.
    pub weight: f32,
}

/// The stateful execution engine holding the graph structure.
#[derive(Clone)]
pub struct QianjiEngine {
    /// The underlying petgraph structure.
    pub graph: StableGraph<QianjiNode, QianjiEdge, Directed>,
}

impl QianjiEngine {
    /// Creates an empty engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
        }
    }

    /// Adds a mechanism to the graph without consensus policy.
    pub fn add_mechanism(&mut self, id: &str, mechanism: Arc<dyn QianjiMechanism>) -> NodeIndex {
        self.add_mechanism_with_consensus(id, mechanism, None)
    }

    /// Adds a mechanism to the graph with optional consensus policy.
    pub fn add_mechanism_with_consensus(
        &mut self,
        id: &str,
        mechanism: Arc<dyn QianjiMechanism>,
        consensus: Option<ConsensusPolicy>,
    ) -> NodeIndex {
        self.add_mechanism_with_affinity(id, mechanism, consensus, NodeExecutionAffinity::default())
    }

    /// Adds a mechanism with optional consensus policy and execution affinity.
    pub fn add_mechanism_with_affinity(
        &mut self,
        id: &str,
        mechanism: Arc<dyn QianjiMechanism>,
        consensus: Option<ConsensusPolicy>,
        execution_affinity: NodeExecutionAffinity,
    ) -> NodeIndex {
        self.graph.add_node(QianjiNode {
            id: id.to_string(),
            status: NodeStatus::Idle,
            mechanism,
            consensus,
            execution_affinity,
        })
    }

    /// Adds a directional link between mechanisms.
    pub fn add_link(&mut self, from: NodeIndex, to: NodeIndex, label: Option<&str>, weight: f32) {
        self.graph.add_edge(
            from,
            to,
            QianjiEdge {
                label: label.map(str::to_string),
                weight,
            },
        );
    }
}

impl Default for QianjiEngine {
    fn default() -> Self {
        Self::new()
    }
}
