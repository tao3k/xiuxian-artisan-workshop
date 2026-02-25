use super::types::{ContextRepairResult, ReactConversationState, TurnRuntimeContext};
#[allow(clippy::wildcard_imports)]
use super::*;

impl Agent {
    pub(super) async fn repair_context_window_and_retry(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
        error: anyhow::Error,
    ) -> Result<ContextRepairResult> {
        let error_text = format!("{error:#}");
        let context_limit_hint = parse_context_window_limit_hint(&error_text);
        tracing::warn!(
            event = "agent.llm.context_window.repair.start",
            session_id = turn_ctx.session_id,
            round = state.round,
            context_limit_hint,
            tools_enabled = state.tools_json.is_some(),
            "llm context window exceeded; starting automatic context repair"
        );

        let mut last_context_error = error;

        if let Some(repaired) = self
            .try_context_repair_drop_tools(turn_ctx, state, &mut last_context_error)
            .await?
        {
            return Ok(repaired);
        }

        if let Some(repaired) = self
            .try_context_repair_with_pruned_budgets(
                turn_ctx,
                state,
                context_limit_hint,
                &mut last_context_error,
            )
            .await?
        {
            return Ok(repaired);
        }

        tracing::error!(
            event = "agent.llm.context_window.repair.failed",
            session_id = turn_ctx.session_id,
            round = state.round,
            context_limit_hint,
            "llm context repair exhausted all retries"
        );
        Err(last_context_error)
    }

    async fn try_context_repair_drop_tools(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
        last_context_error: &mut anyhow::Error,
    ) -> Result<Option<ContextRepairResult>> {
        if state.tools_json.is_none() {
            return Ok(None);
        }

        match self.llm.chat(state.messages.clone(), None).await {
            Ok(response) => {
                tracing::warn!(
                    event = "agent.llm.context_window.repair.success",
                    session_id = turn_ctx.session_id,
                    round = state.round,
                    strategy = "drop_tools_only",
                    "llm context repair succeeded by dropping tools payload"
                );
                Ok(Some(ContextRepairResult {
                    response,
                    messages: state.messages.clone(),
                    tools_json: None,
                }))
            }
            Err(retry_error) if is_context_window_exceeded_error(&retry_error) => {
                *last_context_error = retry_error;
                Ok(None)
            }
            Err(retry_error) => Err(retry_error),
        }
    }

    async fn try_context_repair_with_pruned_budgets(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
        context_limit_hint: Option<usize>,
        last_context_error: &mut anyhow::Error,
    ) -> Result<Option<ContextRepairResult>> {
        for budget in context_window_recovery_budgets(context_limit_hint) {
            let pruned = context_budget::prune_messages_for_token_budget_with_strategy(
                state.messages.clone(),
                budget,
                0,
                self.config.context_budget_strategy,
            )
            .messages;
            if pruned.is_empty() {
                continue;
            }

            if let Some(repaired) = self
                .try_pruned_repair_with_tools(turn_ctx, state, budget, &pruned, last_context_error)
                .await?
            {
                return Ok(Some(repaired));
            }

            if let Some(repaired) = self
                .try_pruned_repair_without_tools(
                    turn_ctx,
                    state,
                    budget,
                    pruned,
                    last_context_error,
                )
                .await?
            {
                return Ok(Some(repaired));
            }
        }

        Ok(None)
    }

    async fn try_pruned_repair_with_tools(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
        budget: usize,
        pruned: &[ChatMessage],
        last_context_error: &mut anyhow::Error,
    ) -> Result<Option<ContextRepairResult>> {
        let Some(tools_payload) = state.tools_json.clone() else {
            return Ok(None);
        };

        match self
            .llm
            .chat(pruned.to_vec(), Some(tools_payload.clone()))
            .await
        {
            Ok(response) => {
                tracing::warn!(
                    event = "agent.llm.context_window.repair.success",
                    session_id = turn_ctx.session_id,
                    round = state.round,
                    strategy = "prune_keep_tools",
                    repair_budget_tokens = budget,
                    "llm context repair succeeded with pruned messages (tools kept)"
                );
                Ok(Some(ContextRepairResult {
                    response,
                    messages: pruned.to_vec(),
                    tools_json: Some(tools_payload),
                }))
            }
            Err(retry_error) if is_context_window_exceeded_error(&retry_error) => {
                *last_context_error = retry_error;
                Ok(None)
            }
            Err(retry_error) => Err(retry_error),
        }
    }

    async fn try_pruned_repair_without_tools(
        &self,
        turn_ctx: &TurnRuntimeContext<'_>,
        state: &ReactConversationState,
        budget: usize,
        pruned: Vec<ChatMessage>,
        last_context_error: &mut anyhow::Error,
    ) -> Result<Option<ContextRepairResult>> {
        match self.llm.chat(pruned.clone(), None).await {
            Ok(response) => {
                tracing::warn!(
                    event = "agent.llm.context_window.repair.success",
                    session_id = turn_ctx.session_id,
                    round = state.round,
                    strategy = "prune_drop_tools",
                    repair_budget_tokens = budget,
                    "llm context repair succeeded with pruned messages and tools disabled"
                );
                Ok(Some(ContextRepairResult {
                    response,
                    messages: pruned,
                    tools_json: None,
                }))
            }
            Err(retry_error) if is_context_window_exceeded_error(&retry_error) => {
                *last_context_error = retry_error;
                Ok(None)
            }
            Err(retry_error) => Err(retry_error),
        }
    }
}

pub(super) fn is_context_window_exceeded_error(error: &anyhow::Error) -> bool {
    let lower = format!("{error:#}").to_ascii_lowercase();
    lower.contains("context window exceeds limit")
        || lower.contains("maximum context length")
        || lower.contains("context_length_exceeded")
        || lower.contains("prompt is too long")
        || lower.contains("context limit")
}

fn parse_context_window_limit_hint(error_text: &str) -> Option<usize> {
    let lower = error_text.to_ascii_lowercase();
    let mut cursor = 0usize;
    while let Some(offset) = lower[cursor..].find("limit") {
        let start = cursor + offset + "limit".len();
        let tail = &lower[start..];
        let digits: String = tail
            .chars()
            .skip_while(|ch| !ch.is_ascii_digit())
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(value) = digits.parse::<usize>() {
            return Some(value);
        }
        cursor = start;
    }
    None
}

fn context_window_recovery_budgets(limit_hint: Option<usize>) -> Vec<usize> {
    let mut budgets = if let Some(limit) = limit_hint {
        vec![
            limit.saturating_mul(3) / 5,
            limit.saturating_mul(1) / 2,
            limit.saturating_mul(2) / 5,
            limit.saturating_mul(1) / 3,
            limit.saturating_mul(1) / 4,
        ]
    } else {
        vec![1024, 768, 512, 384, 256]
    };
    budgets.retain(|budget| *budget > 0);
    budgets.sort_unstable();
    budgets.dedup();
    budgets.reverse();
    budgets
}
