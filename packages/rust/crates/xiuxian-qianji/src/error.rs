//! Unified error handling for the Qianji Engine.

use thiserror::Error;

/// Error types emitted during graph compilation or execution.
#[derive(Error, Debug, Clone)]
pub enum QianjiError {
    /// Failure during graph topology verification or compilation.
    #[error("Graph topology error: {0}")]
    Topology(String),

    /// Failure during node execution.
    #[error("Node execution failed: {0}")]
    Execution(String),

    /// Strategic deviation detected during auditing.
    #[error("Strategic drift detected: {0}")]
    Drift(String),

    /// Internal capacity or resource limit reached.
    #[error("Resource exhaustion: {0}")]
    Capacity(String),

    /// Checkpoint persistence or retrieval failure.
    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    /// Execution interrupted by swarm-wide cancellation.
    #[error("Execution aborted: {0}")]
    Aborted(String),
}
