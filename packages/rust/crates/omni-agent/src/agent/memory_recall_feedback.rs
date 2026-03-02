use super::memory_recall::MemoryRecallPlan;
use omni_memory::{
    RecallFeedbackOutcome, RecallPlanTuning, apply_feedback_to_plan_tuning,
    update_feedback_bias as update_feedback_bias_model,
};

const FAILURE_KEYWORDS: [&str; 10] = [
    "error",
    "failed",
    "failure",
    "exception",
    "traceback",
    "timeout",
    "timed out",
    "panic",
    "unavailable",
    "invalid",
];

const FEEDBACK_SUCCESS_KEYWORDS: [&str; 8] =
    ["success", "succeeded", "good", "ok", "pass", "up", "+", "1"];
const FEEDBACK_FAILURE_KEYWORDS: [&str; 8] =
    ["failure", "failed", "bad", "error", "down", "-", "0", "no"];

pub(super) const RECALL_FEEDBACK_SOURCE_USER: &str = "user_feedback";
pub(super) const RECALL_FEEDBACK_SOURCE_TOOL: &str = "tool_execution";
pub(super) const RECALL_FEEDBACK_SOURCE_ASSISTANT: &str = "assistant_heuristic";
pub(super) const RECALL_FEEDBACK_SOURCE_COMMAND: &str = "session_feedback_command";

pub(super) type RecallOutcome = RecallFeedbackOutcome;

pub(super) fn update_feedback_bias(previous: f32, outcome: RecallOutcome) -> f32 {
    update_feedback_bias_model(previous, outcome)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ToolExecutionSummary {
    pub(super) attempted: u32,
    pub(super) succeeded: u32,
    pub(super) failed: u32,
}

impl ToolExecutionSummary {
    pub(super) fn record_result(&mut self, is_error: bool) {
        self.attempted = self.attempted.saturating_add(1);
        if is_error {
            self.failed = self.failed.saturating_add(1);
        } else {
            self.succeeded = self.succeeded.saturating_add(1);
        }
    }

    pub(super) fn record_transport_failure(&mut self) {
        self.attempted = self.attempted.saturating_add(1);
        self.failed = self.failed.saturating_add(1);
    }

    pub(super) fn inferred_outcome(self) -> Option<RecallOutcome> {
        if self.attempted == 0 {
            return None;
        }
        if self.failed > 0 && self.succeeded == 0 {
            return Some(RecallOutcome::Failure);
        }
        if self.succeeded > 0 && self.failed == 0 {
            return Some(RecallOutcome::Success);
        }
        None
    }
}

pub(super) fn classify_assistant_outcome(message: &str) -> RecallOutcome {
    let lower = message.to_lowercase();
    if FAILURE_KEYWORDS
        .iter()
        .any(|keyword| lower.contains(keyword))
    {
        RecallOutcome::Failure
    } else {
        RecallOutcome::Success
    }
}

pub(super) fn parse_explicit_user_feedback(message: &str) -> Option<RecallOutcome> {
    let normalized = message.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    if let Some(rest) = normalized.strip_prefix("/feedback") {
        return parse_feedback_suffix(rest);
    }
    if let Some(rest) = normalized.strip_prefix("feedback:") {
        return parse_feedback_suffix(rest);
    }
    if normalized.starts_with("[feedback:") && normalized.ends_with(']') {
        let body = &normalized["[feedback:".len()..normalized.len().saturating_sub(1)];
        return parse_feedback_suffix(body);
    }
    None
}

pub(super) fn resolve_feedback_outcome(
    user_message: &str,
    tool_summary: Option<&ToolExecutionSummary>,
    assistant_message: &str,
) -> (RecallOutcome, &'static str) {
    if let Some(outcome) = parse_explicit_user_feedback(user_message) {
        return (outcome, RECALL_FEEDBACK_SOURCE_USER);
    }
    if let Some(outcome) = tool_summary.and_then(|summary| summary.inferred_outcome()) {
        return (outcome, RECALL_FEEDBACK_SOURCE_TOOL);
    }
    (
        classify_assistant_outcome(assistant_message),
        RECALL_FEEDBACK_SOURCE_ASSISTANT,
    )
}

pub(super) fn apply_feedback_to_plan(
    mut plan: MemoryRecallPlan,
    feedback_bias: f32,
) -> MemoryRecallPlan {
    let tuned = apply_feedback_to_plan_tuning(
        RecallPlanTuning {
            k1: plan.k1,
            k2: plan.k2,
            lambda: plan.lambda,
            min_score: plan.min_score,
            max_context_chars: plan.max_context_chars,
        },
        feedback_bias,
    );
    plan.k1 = tuned.k1.max(1);
    plan.k2 = tuned.k2.max(1).min(plan.k1);
    plan.lambda = tuned.lambda;
    plan.min_score = tuned.min_score;
    plan.max_context_chars = tuned.max_context_chars;
    plan
}

fn parse_feedback_suffix(raw: &str) -> Option<RecallOutcome> {
    let token = raw
        .trim()
        .trim_start_matches([':', '='])
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(|c: char| c == '"' || c == '\'' || c == ',' || c == ';');
    parse_feedback_token(token)
}

fn parse_feedback_token(token: &str) -> Option<RecallOutcome> {
    if token.is_empty() {
        return None;
    }
    if FEEDBACK_SUCCESS_KEYWORDS.contains(&token) {
        return Some(RecallOutcome::Success);
    }
    if FEEDBACK_FAILURE_KEYWORDS.contains(&token) {
        return Some(RecallOutcome::Failure);
    }
    None
}
