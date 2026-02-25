use super::super::types::{
    MemoryRecallExecutionContext, MemoryRecallPlanContext, MemoryRecallResultStats,
};
#[allow(clippy::wildcard_imports)]
use super::super::*;

use crate::agent::embedding_runtime::MemoryEmbeddingErrorKind;

impl Agent {
    pub(super) async fn record_injected_memory_recall(
        &self,
        recall_ctx: &MemoryRecallPlanContext,
        execution_ctx: &MemoryRecallExecutionContext<'_>,
        stats: MemoryRecallResultStats,
    ) {
        tracing::debug!(
            event = SessionEvent::MemoryRecallInjected.as_str(),
            session_id = execution_ctx.session_id,
            query_tokens = recall_ctx.query_tokens,
            embedding_source = execution_ctx.embedding_source,
            recalled_total = stats.recalled_count,
            recalled_selected = stats.selected_count,
            recalled_injected = stats.injected_count,
            context_chars_injected = stats.context_chars_injected,
            pipeline_duration_ms = stats.pipeline_duration_ms,
            best_score = stats.best_score.unwrap_or_default(),
            weakest_score = stats.weakest_score.unwrap_or_default(),
            "memory recall context injected"
        );
        self.record_memory_recall_result_metrics(
            memory_recall_state::SessionMemoryRecallDecision::Injected,
            stats.selected_count,
            stats.injected_count,
            stats.context_chars_injected,
            stats.pipeline_duration_ms,
        )
        .await;
        self.record_memory_recall_snapshot(
            execution_ctx.session_id,
            memory_recall_state::SessionMemoryRecallSnapshot::from_plan(
                recall_ctx.recall_plan,
                memory_recall_state::SessionMemoryRecallSnapshotInput {
                    active_turns_estimate: recall_ctx.active_turns_estimate,
                    summary_segment_count: execution_ctx.summary_segment_count,
                    query_tokens: recall_ctx.query_tokens,
                    recall_feedback_bias: recall_ctx.recall_feedback_bias,
                    embedding_source: execution_ctx.embedding_source,
                    recalled_total: stats.recalled_count,
                    recalled_selected: stats.selected_count,
                    recalled_injected: stats.injected_count,
                    context_chars_injected: stats.context_chars_injected,
                    best_score: stats.best_score,
                    weakest_score: stats.weakest_score,
                    pipeline_duration_ms: stats.pipeline_duration_ms,
                    decision: memory_recall_state::SessionMemoryRecallDecision::Injected,
                },
            ),
        )
        .await;
    }

    pub(super) async fn record_skipped_memory_recall(
        &self,
        recall_ctx: &MemoryRecallPlanContext,
        execution_ctx: &MemoryRecallExecutionContext<'_>,
        stats: MemoryRecallResultStats,
    ) {
        tracing::debug!(
            event = SessionEvent::MemoryRecallSkipped.as_str(),
            session_id = execution_ctx.session_id,
            query_tokens = recall_ctx.query_tokens,
            embedding_source = execution_ctx.embedding_source,
            recalled_total = stats.recalled_count,
            recalled_selected = stats.selected_count,
            pipeline_duration_ms = stats.pipeline_duration_ms,
            best_score = stats.best_score.unwrap_or_default(),
            "memory recall skipped after scoring/compaction filters"
        );
        self.record_memory_recall_result_metrics(
            memory_recall_state::SessionMemoryRecallDecision::Skipped,
            stats.selected_count,
            0,
            0,
            stats.pipeline_duration_ms,
        )
        .await;
        self.record_memory_recall_snapshot(
            execution_ctx.session_id,
            memory_recall_state::SessionMemoryRecallSnapshot::from_plan(
                recall_ctx.recall_plan,
                memory_recall_state::SessionMemoryRecallSnapshotInput {
                    active_turns_estimate: recall_ctx.active_turns_estimate,
                    summary_segment_count: execution_ctx.summary_segment_count,
                    query_tokens: recall_ctx.query_tokens,
                    recall_feedback_bias: recall_ctx.recall_feedback_bias,
                    embedding_source: execution_ctx.embedding_source,
                    recalled_total: stats.recalled_count,
                    recalled_selected: stats.selected_count,
                    recalled_injected: 0,
                    context_chars_injected: 0,
                    best_score: stats.best_score,
                    weakest_score: stats.weakest_score,
                    pipeline_duration_ms: stats.pipeline_duration_ms,
                    decision: memory_recall_state::SessionMemoryRecallDecision::Skipped,
                },
            ),
        )
        .await;
    }

    pub(super) async fn record_memory_recall_embedding_failure(
        &self,
        recall_ctx: &MemoryRecallPlanContext,
        session_id: &str,
        summary_segment_count: usize,
        error_kind: MemoryEmbeddingErrorKind,
    ) {
        let pipeline_duration_ms =
            u64::try_from(recall_ctx.recall_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::warn!(
            event = SessionEvent::MemoryRecallSkipped.as_str(),
            session_id,
            query_tokens = recall_ctx.query_tokens,
            reason = error_kind.as_str(),
            pipeline_duration_ms,
            "memory recall skipped because embedding request failed"
        );
        self.record_memory_recall_result_metrics(
            memory_recall_state::SessionMemoryRecallDecision::Skipped,
            0,
            0,
            0,
            pipeline_duration_ms,
        )
        .await;
        self.record_memory_recall_snapshot(
            session_id,
            memory_recall_state::SessionMemoryRecallSnapshot::from_plan(
                recall_ctx.recall_plan,
                memory_recall_state::SessionMemoryRecallSnapshotInput {
                    active_turns_estimate: recall_ctx.active_turns_estimate,
                    summary_segment_count,
                    query_tokens: recall_ctx.query_tokens,
                    recall_feedback_bias: recall_ctx.recall_feedback_bias,
                    embedding_source: EMBEDDING_SOURCE_UNAVAILABLE,
                    recalled_total: 0,
                    recalled_selected: 0,
                    recalled_injected: 0,
                    context_chars_injected: 0,
                    best_score: None,
                    weakest_score: None,
                    pipeline_duration_ms,
                    decision: memory_recall_state::SessionMemoryRecallDecision::Skipped,
                },
            ),
        )
        .await;
    }
}
