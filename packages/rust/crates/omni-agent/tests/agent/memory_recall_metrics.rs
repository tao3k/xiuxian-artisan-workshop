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

use super::{MemoryRecallMetricsState, ratio_as_f32};
use crate::agent::SessionMemoryRecallDecision;

#[test]
fn ratio_as_f32_handles_zero_denominator() {
    assert_eq!(ratio_as_f32(10, 0), 0.0);
    assert_eq!(ratio_as_f32(9, 3), 3.0);
}

#[test]
fn metrics_snapshot_aggregates_counts_and_rates() {
    let mut state = MemoryRecallMetricsState::default();
    state.observe_plan();
    state.observe_plan();
    state.observe_plan();

    state.observe_result(SessionMemoryRecallDecision::Injected, 3, 2, 420, 18);
    state.observe_result(SessionMemoryRecallDecision::Skipped, 1, 0, 0, 260);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.planned_total, 3);
    assert_eq!(snapshot.injected_total, 1);
    assert_eq!(snapshot.skipped_total, 1);
    assert_eq!(snapshot.completed_total, 2);
    assert_eq!(snapshot.selected_total, 4);
    assert_eq!(snapshot.injected_items_total, 2);
    assert_eq!(snapshot.context_chars_injected_total, 420);
    assert_eq!(snapshot.pipeline_duration_ms_total, 278);
    assert!((snapshot.avg_pipeline_duration_ms - 139.0).abs() < 0.001);
    assert!((snapshot.avg_selected_per_completed - 2.0).abs() < 0.001);
    assert!((snapshot.avg_injected_per_injected - 2.0).abs() < 0.001);
    assert!((snapshot.injected_rate - 0.5).abs() < 0.001);
    assert_eq!(snapshot.embedding_success_total, 0);
    assert_eq!(snapshot.embedding_timeout_total, 0);
    assert_eq!(snapshot.embedding_cooldown_reject_total, 0);
    assert_eq!(snapshot.embedding_unavailable_total, 0);
}

#[test]
fn metrics_latency_buckets_are_classified_deterministically() {
    let mut state = MemoryRecallMetricsState::default();

    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 10);
    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 25);
    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 50);
    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 100);
    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 250);
    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 500);
    state.observe_result(SessionMemoryRecallDecision::Injected, 1, 1, 10, 900);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.latency_buckets.le_10ms, 1);
    assert_eq!(snapshot.latency_buckets.le_25ms, 1);
    assert_eq!(snapshot.latency_buckets.le_50ms, 1);
    assert_eq!(snapshot.latency_buckets.le_100ms, 1);
    assert_eq!(snapshot.latency_buckets.le_250ms, 1);
    assert_eq!(snapshot.latency_buckets.le_500ms, 1);
    assert_eq!(snapshot.latency_buckets.gt_500ms, 1);
}

#[test]
fn metrics_snapshot_tracks_embedding_outcome_counters() {
    let mut state = MemoryRecallMetricsState::default();

    state.observe_embedding_success();
    state.observe_embedding_success();
    state.observe_embedding_timeout();
    state.observe_embedding_cooldown_reject();
    state.observe_embedding_cooldown_reject();
    state.observe_embedding_unavailable();

    let snapshot = state.snapshot();
    assert_eq!(snapshot.embedding_success_total, 2);
    assert_eq!(snapshot.embedding_timeout_total, 1);
    assert_eq!(snapshot.embedding_cooldown_reject_total, 2);
    assert_eq!(snapshot.embedding_unavailable_total, 1);
}
