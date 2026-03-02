//! Omega: Strategic routing and quality-gating engine.

mod decision;

pub(crate) use decision::{apply_policy_hint, apply_quality_gate, decide_for_standard_turn};
