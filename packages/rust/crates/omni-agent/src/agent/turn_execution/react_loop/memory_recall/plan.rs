use super::super::types::{MemoryRecallPlanContext, MemoryRecallTuning};
#[allow(clippy::wildcard_imports)]
use super::super::*;

impl Agent {
    pub(super) async fn build_memory_recall_plan_context(
        &self,
        session_id: &str,
        user_message: &str,
        messages: &[ChatMessage],
        summary_segment_count: usize,
        recall_tuning: MemoryRecallTuning,
    ) -> MemoryRecallPlanContext {
        let recall_started = Instant::now();
        let active_turns_estimate = messages
            .iter()
            .filter(|message| message.role == "user" || message.role == "assistant")
            .count()
            / 2;
        let query_tokens = count_tokens(user_message);
        let recall_plan = plan_memory_recall(MemoryRecallInput {
            base_k1: recall_tuning.k1,
            base_k2: recall_tuning.k2,
            base_lambda: recall_tuning.lambda,
            context_budget_tokens: self.config.context_budget_tokens,
            context_budget_reserve_tokens: self.config.context_budget_reserve_tokens,
            context_tokens_before_recall: estimate_messages_tokens(messages),
            active_turns_estimate,
            window_max_turns: self.config.window_max_turns,
            summary_segment_count,
        });
        let recall_feedback_bias = self.recall_feedback_bias(session_id).await;
        let recall_plan = apply_feedback_to_plan(recall_plan, recall_feedback_bias);

        tracing::debug!(
            event = SessionEvent::MemoryRecallPlanned.as_str(),
            session_id,
            memory_scope = session_id,
            k1 = recall_plan.k1,
            k2 = recall_plan.k2,
            lambda = recall_plan.lambda,
            min_score = recall_plan.min_score,
            max_context_chars = recall_plan.max_context_chars,
            budget_pressure = recall_plan.budget_pressure,
            window_pressure = recall_plan.window_pressure,
            effective_budget_tokens = ?recall_plan.effective_budget_tokens,
            active_turns_estimate,
            summary_segment_count,
            recall_feedback_bias,
            "memory recall plan selected"
        );

        MemoryRecallPlanContext {
            recall_started,
            active_turns_estimate,
            query_tokens,
            recall_plan,
            recall_feedback_bias,
        }
    }
}
