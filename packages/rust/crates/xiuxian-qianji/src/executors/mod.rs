//! Built-in node execution mechanisms for the Qianji Box.

/// Context annotation mechanism.
pub mod annotation;
/// Adversarial calibration mechanism (Synapse-Audit).
pub mod calibration;
/// Local shell command executor.
pub mod command;
/// Formal LTL audit mechanism.
pub mod formal_audit;
/// Wendao knowledge retrieval mechanism.
pub mod knowledge;
/// Mock mechanism for testing.
pub mod mock;
/// Probabilistic MDP routing mechanism.
pub mod router;
/// AST-based Security Scanning Mechanism.
pub mod security_scan;
/// Workflow suspension for human-in-the-loop.
pub mod suspend;
/// Native memory-promotion ingestion into `Wendao`.
pub mod wendao_ingester;
/// Incremental-first LinkGraph refresh trigger for Wendao.
pub mod wendao_refresh;
/// Native file writing mechanism with parent-directory bootstrap.
pub mod write_file;

#[cfg(feature = "llm")]
/// LLM analysis mechanism.
pub mod llm;

pub use mock::MockMechanism;
