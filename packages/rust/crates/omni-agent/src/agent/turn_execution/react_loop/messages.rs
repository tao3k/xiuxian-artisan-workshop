use anyhow::Result;

use super::types::ReactPreparedMessages;
use crate::agent::system_prompt_injection_state::SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME;
use crate::agent::{Agent, context_budget, injection};
use crate::session::{ChatMessage, SessionSummarySegment};
use xiuxian_qianhuan::InjectionPolicy;
use xiuxian_zhenfa::ZhenfaTransmuter;

impl Agent {
    pub(super) async fn prepare_react_messages(
        &self,
        session_id: &str,
        user_message: &str,
    ) -> Result<ReactPreparedMessages> {
        let mut summary_segments: Vec<SessionSummarySegment> = Vec::new();
        let mut messages: Vec<ChatMessage> = if let Some(ref w) = self.bounded_session {
            let limit = self.config.window_max_turns.unwrap_or(512);
            let (recent_summary_segments, recent_messages) = tokio::try_join!(
                w.get_recent_summary_segments(session_id, self.config.summary_max_segments),
                w.get_recent_messages(session_id, limit)
            )?;
            summary_segments = recent_summary_segments;
            recent_messages
        } else {
            self.session.get(session_id).await?
        };

        Self::prepend_summary_segments(&mut messages, &summary_segments);

        // NATIVE: Inject a clear summary of built-in capabilities to the LLM
        let native_summary = self.native_tools.get_registry_summary();
        messages.insert(
            0,
            ChatMessage {
                role: "system".to_string(),
                content: Some(native_summary),
                tool_calls: None,
                tool_call_id: None,
                name: Some("native_tools_summary".to_string()),
            },
        );

        if let Some(snapshot) = self
            .inspect_session_system_prompt_injection(session_id)
            .await
        {
            messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: Some(snapshot.xml),
                    tool_calls: None,
                    tool_call_id: None,
                    name: Some(SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME.to_string()),
                },
            );
        }

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: Some(user_message.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        let messages = Self::transmute_messages_for_llm(session_id, messages);

        Ok(ReactPreparedMessages {
            messages,
            summary_segment_count: summary_segments.len(),
        })
    }

    pub(super) fn prepend_summary_segments(
        messages: &mut Vec<ChatMessage>,
        summary_segments: &[SessionSummarySegment],
    ) {
        if summary_segments.is_empty() {
            return;
        }
        let segment_count = summary_segments.len();
        let summary_messages = summary_segments
            .iter()
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
            .collect::<Vec<_>>();
        messages.splice(0..0, summary_messages);
    }

    pub(super) async fn normalize_and_pack_react_messages(
        &self,
        session_id: &str,
        turn_id: u64,
        messages: Vec<ChatMessage>,
    ) -> Vec<ChatMessage> {
        let normalized = injection::normalize_messages_with_snapshot(
            session_id,
            turn_id,
            messages,
            InjectionPolicy::default(),
        );
        if let Some(snapshot) = normalized.snapshot.as_ref() {
            Self::record_injection_snapshot(session_id, snapshot);
        }
        let mut messages = normalized.messages;

        if let Some(context_budget_tokens) = self.config.context_budget_tokens
            && context_budget_tokens > 0
        {
            let result = context_budget::prune_messages_for_token_budget_with_strategy(
                messages,
                context_budget_tokens,
                self.config.context_budget_reserve_tokens,
                self.config.context_budget_strategy,
            );
            messages = result.messages;
            let report = result.report;
            self.record_context_budget_snapshot(session_id, &report)
                .await;
            tracing::debug!(
                session_id,
                strategy = report.strategy.as_str(),
                budget_tokens = report.budget_tokens,
                reserve_tokens = report.reserve_tokens,
                effective_budget_tokens = report.effective_budget_tokens,
                pre_messages = report.pre_messages,
                post_messages = report.post_messages,
                pre_tokens = report.pre_tokens,
                post_tokens = report.post_tokens,
                dropped_messages = report.pre_messages.saturating_sub(report.post_messages),
                dropped_tokens = report.pre_tokens.saturating_sub(report.post_tokens),
                non_system_pre_messages = report.non_system.input_messages,
                non_system_kept_messages = report.non_system.kept_messages,
                non_system_dropped_messages = report.non_system.dropped_messages(),
                non_system_pre_tokens = report.non_system.input_tokens,
                non_system_kept_tokens = report.non_system.kept_tokens,
                non_system_dropped_tokens = report.non_system.dropped_tokens(),
                non_system_truncated_messages = report.non_system.truncated_messages,
                non_system_truncated_tokens = report.non_system.truncated_tokens,
                regular_system_pre_messages = report.regular_system.input_messages,
                regular_system_kept_messages = report.regular_system.kept_messages,
                regular_system_dropped_messages = report.regular_system.dropped_messages(),
                regular_system_pre_tokens = report.regular_system.input_tokens,
                regular_system_kept_tokens = report.regular_system.kept_tokens,
                regular_system_dropped_tokens = report.regular_system.dropped_tokens(),
                regular_system_truncated_messages = report.regular_system.truncated_messages,
                regular_system_truncated_tokens = report.regular_system.truncated_tokens,
                summary_pre_messages = report.summary_system.input_messages,
                summary_kept_messages = report.summary_system.kept_messages,
                summary_dropped_messages = report.summary_system.dropped_messages(),
                summary_pre_tokens = report.summary_system.input_tokens,
                summary_kept_tokens = report.summary_system.kept_tokens,
                summary_dropped_tokens = report.summary_system.dropped_tokens(),
                summary_truncated_messages = report.summary_system.truncated_messages,
                summary_truncated_tokens = report.summary_system.truncated_tokens,
                "applied token-budget context packing"
            );
        }

        messages
    }

    pub(super) async fn load_tools_json_for_react(&self) -> Result<Option<Vec<serde_json::Value>>> {
        if self.mcp.is_some() {
            self.mcp_tools_for_llm().await
        } else {
            Ok(None)
        }
    }

    fn transmute_messages_for_llm(
        session_id: &str,
        messages: Vec<ChatMessage>,
    ) -> Vec<ChatMessage> {
        let mut sealed_messages = Vec::with_capacity(messages.len());
        for mut message in messages {
            let Some(content) = message.content.take() else {
                sealed_messages.push(message);
                continue;
            };
            match ZhenfaTransmuter::validate_and_refine(content.as_str()) {
                Ok(refined) => {
                    message.content = Some(refined);
                }
                Err(error) => {
                    let fallback = ZhenfaTransmuter::refine_for_llm(content.as_str());
                    tracing::warn!(
                        event = "agent.zhenfa.transmuter.validation_failed",
                        session_id,
                        role = message.role.as_str(),
                        message_name = message.name.as_deref().unwrap_or(""),
                        llm_safe_reason = error.llm_safe_message(),
                        error = %error,
                        "message content failed structural validation; using refined fallback"
                    );
                    message.content = Some(fallback);
                }
            }
            sealed_messages.push(message);
        }
        sealed_messages
    }
}
