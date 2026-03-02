//! Session-level recall feedback model and plan tuning utilities.

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};

/// Feedback outcome used for recall bias adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallFeedbackOutcome {
    /// Recall behavior contributed positively.
    Success,
    /// Recall behavior contributed negatively.
    Failure,
}

impl RecallFeedbackOutcome {
    /// Canonical textual label for telemetry and diagnostics.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }

    /// Canonical memory label.
    #[must_use]
    pub fn as_memory_label(self) -> &'static str {
        match self {
            Self::Success => "completed",
            Self::Failure => "error",
        }
    }

    /// Signed direction delta for feedback adaptation.
    #[must_use]
    pub fn as_feedback_delta(self) -> f32 {
        match self {
            Self::Success => 1.0,
            Self::Failure => -1.0,
        }
    }
}

/// Normalized subset of recall plan fields affected by feedback bias.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecallPlanTuning {
    /// Phase-1 candidate count.
    pub k1: usize,
    /// Phase-2 rerank output count.
    pub k2: usize,
    /// Q-value blending weight in rerank score.
    pub lambda: f32,
    /// Minimum retained similarity score.
    pub min_score: f32,
    /// Max context budget in chars for injected recall block.
    pub max_context_chars: usize,
}

/// Normalize a feedback-bias value to finite `[-1.0, 1.0]`.
#[must_use]
pub fn normalize_feedback_bias(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// Update feedback bias from one outcome.
#[must_use]
pub fn update_feedback_bias(previous: f32, outcome: RecallFeedbackOutcome) -> f32 {
    let previous = normalize_feedback_bias(previous);
    let delta = outcome.as_feedback_delta();
    ((previous * 0.85) + (delta * 0.15)).clamp(-1.0, 1.0)
}

/// Adjust recall tuning with the session feedback bias.
#[must_use]
pub fn apply_feedback_to_plan_tuning(
    mut plan: RecallPlanTuning,
    feedback_bias: f32,
) -> RecallPlanTuning {
    let feedback_bias = normalize_feedback_bias(feedback_bias);
    if feedback_bias <= -0.25 {
        let strength = (-feedback_bias).min(1.0);
        let extra_k2 = if strength >= 0.7 { 2 } else { 1 };
        let extra_k1 = extra_k2 * 3;
        plan.k2 = plan.k2.saturating_add(extra_k2);
        plan.k1 = plan.k1.saturating_add(extra_k1).max(plan.k2);
        plan.lambda = (plan.lambda - (0.06 * strength)).clamp(0.0, 1.0);
        plan.min_score = (plan.min_score - (0.05 * strength)).clamp(0.01, 1.0);
        let context_delta = (240.0 * strength).round().to_usize().unwrap_or(usize::MAX);
        plan.max_context_chars = plan
            .max_context_chars
            .saturating_add(context_delta)
            .clamp(320, 2_400);
    } else if feedback_bias >= 0.35 {
        let strength = feedback_bias.min(1.0);
        let reduce_k2 = if strength >= 0.7 { 2 } else { 1 };
        let reduce_k1 = reduce_k2 * 2;
        plan.k2 = plan.k2.saturating_sub(reduce_k2).max(1);
        plan.k1 = plan.k1.saturating_sub(reduce_k1).max(plan.k2);
        plan.lambda = (plan.lambda + (0.05 * strength)).clamp(0.0, 1.0);
        plan.min_score = (plan.min_score + (0.04 * strength)).clamp(0.01, 0.35);
        let context_delta = (160.0 * strength).round().to_usize().unwrap_or(usize::MAX);
        plan.max_context_chars = plan
            .max_context_chars
            .saturating_sub(context_delta)
            .clamp(320, 2_400);
    }

    plan.k1 = plan.k1.max(1);
    plan.k2 = plan.k2.max(1).min(plan.k1);
    plan
}
