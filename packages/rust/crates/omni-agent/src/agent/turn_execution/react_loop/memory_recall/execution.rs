use crate::agent::Agent;
use crate::session::ChatMessage;

use super::super::types::{
    MemoryRecallExecutionContext, MemoryRecallOutcome, MemoryRecallPlanContext,
    MemoryRecallResultStats, MemoryRecallTuning,
};

use crate::agent::memory::select_recall_credit_candidates;
use crate::agent::memory_recall::{
    MEMORY_RECALL_MESSAGE_NAME, build_memory_context_message, filter_recalled_episodes,
};
use omni_memory::Episode;

impl Agent {
    pub(in super::super) async fn run_memory_recall_if_enabled(
        &self,
        session_id: &str,
        user_message: &str,
        messages: &[ChatMessage],
        summary_segment_count: usize,
    ) -> MemoryRecallOutcome {
        let (Some(store), Some(mem_cfg)) =
            (self.memory_store.as_ref(), self.config.memory.as_ref())
        else {
            return MemoryRecallOutcome {
                system_message: None,
                recall_credit_candidates: Vec::new(),
            };
        };

        let recall_tuning = MemoryRecallTuning {
            k1: mem_cfg.recall_k1,
            k2: mem_cfg.recall_k2,
            lambda: mem_cfg.recall_lambda,
        };
        let recall_ctx = self.build_memory_recall_plan_context(
            session_id,
            user_message,
            messages,
            summary_segment_count,
            recall_tuning,
        );
        self.record_memory_recall_plan_metrics().await;

        match self
            .embedding_for_memory_with_source(user_message, mem_cfg.embedding_dim)
            .await
        {
            Ok((query_embedding, embedding_source)) => {
                let recalled = store.two_phase_recall_with_embedding_for_scope(
                    session_id,
                    &query_embedding,
                    recall_ctx.recall_plan.k1,
                    recall_ctx.recall_plan.k2,
                    recall_ctx.recall_plan.lambda,
                );
                let execution_ctx = MemoryRecallExecutionContext {
                    session_id,
                    summary_segment_count,
                    embedding_source,
                    recall_credit_enabled: mem_cfg.recall_credit_enabled,
                    recall_credit_max_candidates: mem_cfg.recall_credit_max_candidates,
                };
                return self
                    .handle_memory_recall_embedding_success(&recall_ctx, &execution_ctx, recalled)
                    .await;
            }
            Err(error_kind) => {
                self.record_memory_recall_embedding_failure(
                    &recall_ctx,
                    session_id,
                    summary_segment_count,
                    error_kind,
                )
                .await;
            }
        }

        MemoryRecallOutcome {
            system_message: None,
            recall_credit_candidates: Vec::new(),
        }
    }

    async fn handle_memory_recall_embedding_success(
        &self,
        recall_ctx: &MemoryRecallPlanContext,
        execution_ctx: &MemoryRecallExecutionContext<'_>,
        recalled: Vec<(Episode, f32)>,
    ) -> MemoryRecallOutcome {
        let recalled_count = recalled.len();
        let recalled = filter_recalled_episodes(recalled, &recall_ctx.recall_plan);

        if let Some(system_content) =
            build_memory_context_message(&recalled, recall_ctx.recall_plan.max_context_chars)
        {
            let recall_credit_candidates = if execution_ctx.recall_credit_enabled {
                select_recall_credit_candidates(
                    &recalled,
                    execution_ctx.recall_credit_max_candidates,
                )
            } else {
                Vec::new()
            };
            let injected_count = recalled.len();
            let context_chars_injected = system_content.chars().count();
            let pipeline_duration_ms =
                u64::try_from(recall_ctx.recall_started.elapsed().as_millis()).unwrap_or(u64::MAX);
            let best_score = recalled.first().map(|(_, score)| *score);
            let weakest_score = recalled.last().map(|(_, score)| *score);

            let stats = MemoryRecallResultStats {
                recalled_count,
                selected_count: recalled.len(),
                injected_count,
                context_chars_injected,
                best_score,
                weakest_score,
                pipeline_duration_ms,
            };
            self.record_injected_memory_recall(recall_ctx, execution_ctx, stats)
                .await;
            return MemoryRecallOutcome {
                system_message: Some(ChatMessage {
                    role: "system".to_string(),
                    content: Some(system_content),
                    tool_calls: None,
                    tool_call_id: None,
                    name: Some(MEMORY_RECALL_MESSAGE_NAME.to_string()),
                }),
                recall_credit_candidates,
            };
        }

        let pipeline_duration_ms =
            u64::try_from(recall_ctx.recall_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let stats = MemoryRecallResultStats {
            recalled_count,
            selected_count: recalled.len(),
            injected_count: 0,
            context_chars_injected: 0,
            best_score: recalled.first().map(|(_, score)| *score),
            weakest_score: recalled.last().map(|(_, score)| *score),
            pipeline_duration_ms,
        };
        self.record_skipped_memory_recall(recall_ctx, execution_ctx, stats)
            .await;
        MemoryRecallOutcome {
            system_message: None,
            recall_credit_candidates: Vec::new(),
        }
    }
}
