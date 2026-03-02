mod context_repair;
mod conversation;
mod memory_recall;
mod messages;
mod types;

use anyhow::Result;

use crate::agent::reflection::PolicyHintDirective;
use crate::agent::{Agent, omega};
use crate::contracts::{OmegaDecision, OmegaFallbackPolicy};
use crate::observability::SessionEvent;
use crate::session::ChatMessage;

use types::{ReactConversationState, ReactPreparedMessages, TurnRuntimeContext};

const NEXT_TURN_HINT_MESSAGE_NAME: &str = "agent.next_turn_hint";

impl Agent {
    pub(in crate::agent) async fn run_react_loop(
        &self,
        session_id: &str,
        user_message: &str,
        force_react: bool,
        turn_id: u64,
    ) -> Result<String> {
        let ((decision, policy_hint), prepared_messages) = tokio::join!(
            self.prepare_react_decision(session_id, force_react),
            self.prepare_react_messages(session_id, user_message)
        );
        let ReactPreparedMessages {
            mut messages,
            summary_segment_count,
        } = prepared_messages?;

        if let Some(hint) = policy_hint.as_ref() {
            messages.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: Some(render_next_turn_hint_block(hint, &decision)),
                    tool_calls: None,
                    tool_call_id: None,
                    name: Some(NEXT_TURN_HINT_MESSAGE_NAME.to_string()),
                },
            );
        }

        let recall_input_messages = messages.clone();
        let memory_recall_outcome = self
            .run_memory_recall_if_enabled(
                session_id,
                user_message,
                &recall_input_messages,
                summary_segment_count,
            )
            .await;
        if let Some(memory_recall_message) = memory_recall_outcome.system_message {
            messages.insert(0, memory_recall_message);
        }
        let recall_credit_candidates = memory_recall_outcome.recall_credit_candidates;

        let messages = self
            .normalize_and_pack_react_messages(session_id, turn_id, messages)
            .await;
        let tools_json = self.load_tools_json_for_react().await?;
        let mut state = ReactConversationState::new(messages, tools_json);
        let turn_ctx = TurnRuntimeContext {
            session_id,
            user_message,
            turn_id,
            route: decision.route,
            recall_credit_candidates: &recall_credit_candidates,
        };

        self.execute_react_rounds(&turn_ctx, &mut state).await
    }

    pub(super) async fn prepare_react_decision(
        &self,
        session_id: &str,
        force_react: bool,
    ) -> (OmegaDecision, Option<PolicyHintDirective>) {
        let policy_hint = self.take_reflection_policy_hint(session_id).await;
        if let Some(hint) = policy_hint.as_ref() {
            tracing::debug!(
                event = SessionEvent::ReflectionPolicyHintApplied.as_str(),
                session_id,
                source_turn_id = hint.source_turn_id,
                preferred_route = hint.preferred_route.as_str(),
                risk_floor = hint.risk_floor.as_str(),
                fallback_override = hint.fallback_override.map(OmegaFallbackPolicy::as_str),
                tool_trust_class = hint.tool_trust_class.as_str(),
                reason = %hint.reason,
                "reflection policy hint applied to route decision"
            );
        }
        let decision = omega::apply_quality_gate(omega::apply_policy_hint(
            omega::decide_for_standard_turn(force_react),
            policy_hint.as_ref(),
        ));
        Self::record_omega_decision(session_id, &decision, None, None);
        (decision, policy_hint)
    }
}

fn render_next_turn_hint_block(hint: &PolicyHintDirective, decision: &OmegaDecision) -> String {
    let role_mix_profile = if matches!(
        decision.policy_id.as_deref(),
        Some("omega.role_mix.recovery.v1")
    ) {
        "recovery"
    } else {
        "normal"
    };
    format!(
        "[next_turn_hint]\nsource_turn_id={}\npreferred_route={}\nrisk_floor={}\n\
fallback_policy={}\ntool_trust_class={}\nrole_mix_profile={}\nreason={}",
        hint.source_turn_id,
        hint.preferred_route.as_str(),
        hint.risk_floor.as_str(),
        hint.fallback_override
            .map_or("none", OmegaFallbackPolicy::as_str),
        hint.tool_trust_class.as_str(),
        role_mix_profile,
        hint.reason
    )
}
