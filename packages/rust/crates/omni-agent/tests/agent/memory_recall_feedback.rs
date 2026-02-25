#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use super::{
    RECALL_FEEDBACK_SOURCE_ASSISTANT, RECALL_FEEDBACK_SOURCE_TOOL, RECALL_FEEDBACK_SOURCE_USER,
    RecallOutcome, ToolExecutionSummary, apply_feedback_to_plan, classify_assistant_outcome,
    parse_explicit_user_feedback, resolve_feedback_outcome, update_feedback_bias,
};
use crate::agent::memory_recall::MemoryRecallPlan;

fn base_plan() -> MemoryRecallPlan {
    MemoryRecallPlan {
        k1: 12,
        k2: 4,
        lambda: 0.30,
        min_score: 0.10,
        max_context_chars: 900,
        budget_pressure: 0.30,
        window_pressure: 0.40,
        effective_budget_tokens: Some(5_000),
    }
}

#[test]
fn classify_assistant_outcome_detects_failure_keywords() {
    assert_eq!(
        classify_assistant_outcome("Tool call failed with timeout"),
        RecallOutcome::Failure
    );
    assert_eq!(
        classify_assistant_outcome("All checks completed successfully"),
        RecallOutcome::Success
    );
}

#[test]
fn parse_explicit_user_feedback_supports_multiple_forms() {
    assert_eq!(
        parse_explicit_user_feedback("/feedback success"),
        Some(RecallOutcome::Success)
    );
    assert_eq!(
        parse_explicit_user_feedback("feedback: failure"),
        Some(RecallOutcome::Failure)
    );
    assert_eq!(
        parse_explicit_user_feedback("[feedback:up]"),
        Some(RecallOutcome::Success)
    );
    assert_eq!(
        parse_explicit_user_feedback("/feedback down"),
        Some(RecallOutcome::Failure)
    );
    assert_eq!(
        parse_explicit_user_feedback("please continue with no label"),
        None
    );
}

#[test]
fn tool_execution_summary_infers_outcome_only_when_unambiguous() {
    let mut success_only = ToolExecutionSummary::default();
    success_only.record_result(false);
    success_only.record_result(false);
    assert_eq!(
        success_only.inferred_outcome(),
        Some(RecallOutcome::Success)
    );

    let mut failure_only = ToolExecutionSummary::default();
    failure_only.record_result(true);
    assert_eq!(
        failure_only.inferred_outcome(),
        Some(RecallOutcome::Failure)
    );

    let mut mixed = ToolExecutionSummary::default();
    mixed.record_result(false);
    mixed.record_result(true);
    assert_eq!(mixed.inferred_outcome(), None);

    let mut transport_failure = ToolExecutionSummary::default();
    transport_failure.record_transport_failure();
    assert_eq!(
        transport_failure.inferred_outcome(),
        Some(RecallOutcome::Failure)
    );
}

#[test]
fn resolve_feedback_outcome_prioritizes_user_feedback() {
    let mut summary = ToolExecutionSummary::default();
    summary.record_result(false);
    let (outcome, source) =
        resolve_feedback_outcome("/feedback failure", Some(&summary), "all checks passed");
    assert_eq!(outcome, RecallOutcome::Failure);
    assert_eq!(source, RECALL_FEEDBACK_SOURCE_USER);
}

#[test]
fn resolve_feedback_outcome_uses_tool_outcome_before_assistant_text() {
    let mut summary = ToolExecutionSummary::default();
    summary.record_result(true);
    let (outcome, source) = resolve_feedback_outcome("normal user message", Some(&summary), "done");
    assert_eq!(outcome, RecallOutcome::Failure);
    assert_eq!(source, RECALL_FEEDBACK_SOURCE_TOOL);
}

#[test]
fn resolve_feedback_outcome_falls_back_to_assistant_heuristic() {
    let mixed_summary = ToolExecutionSummary {
        attempted: 2,
        succeeded: 1,
        failed: 1,
    };
    let (outcome, source) =
        resolve_feedback_outcome("normal user message", Some(&mixed_summary), "timed out");
    assert_eq!(outcome, RecallOutcome::Failure);
    assert_eq!(source, RECALL_FEEDBACK_SOURCE_ASSISTANT);
}

#[test]
fn update_feedback_bias_moves_toward_failure() {
    let updated = update_feedback_bias(0.2, RecallOutcome::Failure);
    assert!(updated < 0.2);
}

#[test]
fn apply_feedback_to_plan_boosts_recall_after_failures() {
    let plan = apply_feedback_to_plan(base_plan(), -0.8);
    assert!(plan.k1 > 12);
    assert!(plan.k2 > 4);
    assert!(plan.min_score < 0.10);
    assert!(plan.max_context_chars > 900);
}

#[test]
fn apply_feedback_to_plan_tightens_recall_after_successes() {
    let plan = apply_feedback_to_plan(base_plan(), 0.9);
    assert!(plan.k1 < 12);
    assert!(plan.k2 < 4);
    assert!(plan.min_score > 0.10);
    assert!(plan.max_context_chars < 900);
}

#[test]
fn apply_feedback_to_plan_preserves_k_invariants() {
    let mut plan = base_plan();
    plan.k1 = 1;
    plan.k2 = 1;
    let adjusted = apply_feedback_to_plan(plan, -1.0);
    assert!(adjusted.k1 >= 1);
    assert!(adjusted.k2 >= 1);
    assert!(adjusted.k2 <= adjusted.k1);
}
