use anyhow::{Context, Result};
use xiuxian_qianhuan::{InjectionMode, InjectionPolicy, InjectionSnapshot};

use super::super::memory_recall::{
    MEMORY_RECALL_MESSAGE_NAME, MemoryRecallInput, build_memory_context_message,
    estimate_messages_tokens, filter_recalled_episodes, plan_memory_recall,
};
use super::super::memory_recall_feedback::apply_feedback_to_plan;
use super::super::system_prompt_injection_state::SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME;
use super::super::{Agent, context_budget, injection};
use crate::observability::SessionEvent;
use crate::session::ChatMessage;

impl Agent {
    pub(crate) fn record_injection_snapshot(session_id: &str, snapshot: &InjectionSnapshot) {
        let role_mix_profile_id = snapshot
            .role_mix
            .as_ref()
            .map(|profile| profile.profile_id.as_str());
        let role_mix_roles = snapshot
            .role_mix
            .as_ref()
            .map_or(0, |profile| profile.roles.len());
        let injection_mode = match snapshot.policy.mode {
            InjectionMode::Single => "single",
            InjectionMode::Classified => "classified",
            InjectionMode::Hybrid => "hybrid",
        };
        tracing::debug!(
            event = SessionEvent::InjectionSnapshotCreated.as_str(),
            session_id,
            snapshot_id = %snapshot.snapshot_id,
            turn_id = snapshot.turn_id,
            injection_mode,
            blocks = snapshot.blocks.len(),
            total_chars = snapshot.total_chars,
            dropped_blocks = snapshot.dropped_block_ids.len(),
            truncated_blocks = snapshot.truncated_block_ids.len(),
            role_mix_profile_id,
            role_mix_roles,
            "injection snapshot created"
        );
        for block_id in &snapshot.dropped_block_ids {
            tracing::debug!(
                event = SessionEvent::InjectionBlockDropped.as_str(),
                session_id,
                snapshot_id = %snapshot.snapshot_id,
                block_id,
                "injection block dropped"
            );
        }
        for block_id in &snapshot.truncated_block_ids {
            tracing::debug!(
                event = SessionEvent::InjectionBlockTruncated.as_str(),
                session_id,
                snapshot_id = %snapshot.snapshot_id,
                block_id,
                "injection block truncated"
            );
        }
    }

    pub(crate) async fn build_shortcut_injection_snapshot(
        &self,
        session_id: &str,
        turn_id: u64,
        user_message: &str,
    ) -> Result<Option<InjectionSnapshot>> {
        let mut context_messages = self
            .shortcut_summary_messages(session_id)
            .await
            .with_context(|| {
                format!("failed to collect summary context for session: {session_id}")
            })?;

        if let Some(snapshot) = self
            .inspect_session_system_prompt_injection(session_id)
            .await
        {
            context_messages.push(ChatMessage {
                role: "system".to_string(),
                content: Some(snapshot.xml),
                tool_calls: None,
                tool_call_id: None,
                name: Some(SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME.to_string()),
            });
        }

        if let Some(memory_message) = self
            .shortcut_memory_recall_message(session_id, user_message, &context_messages)
            .await
            .with_context(|| {
                format!("failed to prepare memory recall context for session: {session_id}")
            })?
        {
            context_messages.push(memory_message);
        }

        if context_messages.is_empty() {
            return Ok(None);
        }

        let mut policy = InjectionPolicy::default();
        policy.max_chars = policy.max_chars.min(3_500);
        injection::build_snapshot_from_messages(session_id, turn_id, context_messages, policy)
            .map(Some)
            .context("failed to build shortcut injection snapshot")
    }

    async fn shortcut_summary_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let Some(window) = self.bounded_session.as_ref() else {
            return Ok(Vec::new());
        };

        let summary_segments = window
            .get_recent_summary_segments(session_id, self.config.summary_max_segments)
            .await?;
        if summary_segments.is_empty() {
            return Ok(Vec::new());
        }

        let segment_count = summary_segments.len();
        Ok(summary_segments
            .into_iter()
            .enumerate()
            .map(|(index, segment)| ChatMessage {
                role: "system".to_string(),
                content: Some(format!(
                    "Compressed conversation history from older turns (segment {}/{}): {} (turns={}, tools={})",
                    index + 1,
                    segment_count,
                    segment.summary,
                    segment.turn_count,
                    segment.tool_calls
                )),
                tool_calls: None,
                tool_call_id: None,
                name: Some(context_budget::SESSION_SUMMARY_MESSAGE_NAME.to_string()),
            })
            .collect())
    }

    async fn shortcut_memory_recall_message(
        &self,
        session_id: &str,
        user_message: &str,
        context_messages: &[ChatMessage],
    ) -> Result<Option<ChatMessage>> {
        let (Some(store), Some(memory_config)) =
            (self.memory_store.as_ref(), self.config.memory.as_ref())
        else {
            return Ok(None);
        };

        let recall_plan = apply_feedback_to_plan(
            plan_memory_recall(MemoryRecallInput {
                base_k1: memory_config.recall_k1,
                base_k2: memory_config.recall_k2,
                base_lambda: memory_config.recall_lambda,
                context_budget_tokens: self.config.context_budget_tokens,
                context_budget_reserve_tokens: self.config.context_budget_reserve_tokens,
                context_tokens_before_recall: estimate_messages_tokens(context_messages),
                active_turns_estimate: 0,
                window_max_turns: self.config.window_max_turns,
                summary_segment_count: 0,
            }),
            self.recall_feedback_bias(session_id).await,
        );

        match self
            .embedding_for_memory(user_message, memory_config.embedding_dim)
            .await
        {
            Ok(query_embedding) => {
                let recalled = store.two_phase_recall_with_embedding_for_scope(
                    session_id,
                    &query_embedding,
                    recall_plan.k1,
                    recall_plan.k2,
                    recall_plan.lambda,
                );
                let recalled = filter_recalled_episodes(recalled, &recall_plan);
                Ok(
                    build_memory_context_message(&recalled, recall_plan.max_context_chars).map(
                        |system_content| ChatMessage {
                            role: "system".to_string(),
                            content: Some(system_content),
                            tool_calls: None,
                            tool_call_id: None,
                            name: Some(MEMORY_RECALL_MESSAGE_NAME.to_string()),
                        },
                    ),
                )
            }
            Err(error_kind) => {
                tracing::warn!(
                    event = "agent.memory.recall.shortcut.skipped",
                    session_id,
                    reason = error_kind.as_str(),
                    "memory recall skipped during shortcut injection due to embedding failure"
                );
                Ok(None)
            }
        }
    }
}
