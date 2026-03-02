//! Lightweight Formal Logic Engine for Synapse-Audit.
//! Implements predicate checking and trace validation.

use serde::{Deserialize, Serialize};

/// Basic logical proposition extracted from LLM output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposition {
    /// The name of the fact or action (e.g., "`RefinedFact`").
    pub predicate: String,
    /// Whether this proposition carries a valid source reference.
    pub has_grounding: bool,
    /// Confidence level assigned by the Analyzer.
    pub confidence: f32,
}

/// Linear Temporal Logic inspired Invariants.
pub enum Invariant {
    /// Globally: Every proposition must be grounded.
    MustBeGrounded,
    /// Future: Eventually, confidence must reach threshold.
    MinConfidence(f32),
}

impl Invariant {
    /// Validates a trace of propositions against the invariant.
    #[must_use]
    pub fn check(&self, trace: &[Proposition]) -> bool {
        match self {
            Invariant::MustBeGrounded => trace.iter().all(|p| p.has_grounding),
            Invariant::MinConfidence(min) => trace.iter().any(|p| p.confidence >= *min),
        }
    }
}
