mod episode;
mod gate;

use crate::session::ChatMessage;
use anyhow::Result;

use super::super::Agent;
use super::persist_memory_state;
use episode::turn_store_outcome;

struct TurnStoreOutcome {
    label: String,
    reward: f32,
}

struct StoredTurnEpisode {
    id: String,
    source: &'static str,
}

impl Agent {
    pub(in crate::agent) async fn append_turn_to_session(
        &self,
        session_id: &str,
        user_msg: &str,
        assistant_msg: &str,
        tool_count: u32,
    ) -> Result<()> {
        if let Some(ref w) = self.bounded_session {
            w.append_turn(session_id, user_msg, assistant_msg, tool_count)
                .await?;
            self.try_consolidate(session_id).await?;
            self.try_store_turn(session_id, user_msg, assistant_msg, tool_count)
                .await;
            return Ok(());
        }
        let user = ChatMessage {
            role: "user".to_string(),
            content: Some(user_msg.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        let assistant = ChatMessage {
            role: "assistant".to_string(),
            content: Some(assistant_msg.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        self.session
            .append(session_id, vec![user, assistant])
            .await?;
        self.try_store_turn(session_id, user_msg, assistant_msg, tool_count)
            .await;
        Ok(())
    }

    /// When memory is enabled, store the current turn as one episode (intent=user, experience=assistant, outcome=completed/error).
    async fn try_store_turn(
        &self,
        session_id: &str,
        user_msg: &str,
        assistant_msg: &str,
        tool_count: u32,
    ) {
        let Some(ref store) = self.memory_store else {
            return;
        };
        let outcome = turn_store_outcome(assistant_msg);
        let gate_policy = self.memory_gate_policy();
        let Some(stored_episode) = self
            .resolve_turn_episode(
                store,
                session_id,
                user_msg,
                assistant_msg,
                tool_count,
                &outcome,
            )
            .await
        else {
            return;
        };

        store.update_q(&stored_episode.id, outcome.reward);
        let _ = store.record_feedback(&stored_episode.id, outcome.reward > 0.0);
        self.evaluate_turn_memory_gate(
            store,
            session_id,
            &stored_episode,
            tool_count,
            &outcome,
            gate_policy,
        )
        .await;

        persist_memory_state(
            self.memory_state_backend.as_ref(),
            store,
            session_id,
            "turn_store",
        );
        self.maybe_apply_memory_decay(session_id, store);
        self.publish_turn_stored_event(session_id, &stored_episode, &outcome)
            .await;
    }

    async fn publish_turn_stored_event(
        &self,
        session_id: &str,
        stored: &StoredTurnEpisode,
        outcome: &TurnStoreOutcome,
    ) {
        self.publish_memory_stream_event(vec![
            ("kind".to_string(), "turn_stored".to_string()),
            ("session_id".to_string(), session_id.to_string()),
            ("episode_id".to_string(), stored.id.clone()),
            ("episode_source".to_string(), stored.source.to_string()),
            ("outcome".to_string(), outcome.label.clone()),
            ("reward".to_string(), format!("{:.3}", outcome.reward)),
        ])
        .await;
    }
}
