//! Formal logic and safety auditing.

pub mod logic;

use crate::engine::QianjiEngine;
use crate::error::QianjiError;
use petgraph::algo::is_cyclic_directed;

/// Guard responsible for static and structural safety of the Qianji Box.
pub struct QianjiSafetyGuard {
    /// Maximum allowed loop iterations before forced termination.
    pub max_loop_iterations: u32,
}

impl QianjiSafetyGuard {
    /// Creates a new guard.
    #[must_use]
    pub fn new(max_loop_iterations: u32) -> Self {
        Self {
            max_loop_iterations,
        }
    }

    /// Performs a static analysis of the graph topology.
    /// Checks for unauthorized cycles that could lead to infinite execution.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when the topology contains cycles without an exit.
    pub fn audit_topology(&self, engine: &QianjiEngine) -> Result<(), QianjiError> {
        if is_cyclic_directed(&engine.graph) {
            return Err(QianjiError::Topology(format!(
                "Infinite cycle detected without exit condition (max_loop_iterations={})",
                self.max_loop_iterations
            )));
        }
        Ok(())
    }
}
