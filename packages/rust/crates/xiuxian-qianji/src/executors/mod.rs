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

#[cfg(feature = "llm")]
/// LLM analysis mechanism.
pub mod llm;

pub use mock::MockMechanism;
