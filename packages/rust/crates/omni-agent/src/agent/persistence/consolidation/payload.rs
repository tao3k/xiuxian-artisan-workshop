use anyhow::Result;
use omni_memory::{Episode, EpisodeStore};

use crate::observability::SessionEvent;
use crate::session::SessionSummarySegment;

use super::super::super::Agent;
use super::super::super::consolidation::{
    build_consolidated_summary_text, now_unix_ms, summarise_drained_turns,
};
use super::{ConsolidatedEpisode, ConsolidationSummaryPayload};

impl Agent {
    pub(super) async fn build_consolidation_payload(
        &self,
        bounded: &crate::session::BoundedSessionStore,
        session_id: &str,
        drained: Vec<(String, String, u32)>,
    ) -> Result<ConsolidationSummaryPayload> {
        let (intent, experience, outcome) = summarise_drained_turns(&drained);
        let drained_tool_calls: u32 = drained.iter().map(|(_, _, tools)| *tools).sum();
        let drained_slots = drained.len();
        let drained_turns = drained_slots / 2;
        let summary_text = build_consolidated_summary_text(&intent, &experience, &outcome);
        let summary_segment = SessionSummarySegment::new(
            summary_text,
            drained_turns,
            drained_tool_calls,
            now_unix_ms(),
        );
        bounded
            .append_summary_segment(session_id, summary_segment)
            .await?;

        Ok(ConsolidationSummaryPayload {
            intent,
            experience,
            outcome,
            drained_slots,
            drained_turns,
            drained_tool_calls,
        })
    }

    pub(super) async fn build_consolidated_episode(
        &self,
        session_id: &str,
        store: &EpisodeStore,
        payload: &ConsolidationSummaryPayload,
    ) -> Result<Option<ConsolidatedEpisode>> {
        let id = format!("consolidated-{session_id}-{}", now_unix_ms());
        let expected_dim = self
            .config
            .memory
            .as_ref()
            .map_or_else(|| store.encoder().dimension(), |cfg| cfg.embedding_dim);
        let embedding = match self
            .embedding_for_memory(&payload.intent, expected_dim)
            .await
        {
            Ok(embedding) => embedding,
            Err(error_kind) => {
                tracing::warn!(
                    event = SessionEvent::MemoryConsolidationStoreFailed.as_str(),
                    session_id,
                    reason = error_kind.as_str(),
                    "memory consolidation skipped due to embedding failure"
                );
                self.publish_memory_stream_event(vec![
                    (
                        "kind".to_string(),
                        "consolidation_skipped_embedding_failed".to_string(),
                    ),
                    ("session_id".to_string(), session_id.to_string()),
                    ("reason".to_string(), error_kind.as_str().to_string()),
                ])
                .await;
                return Ok(None);
            }
        };
        let reward = consolidation_reward(&payload.outcome);
        let episode = Episode::new(
            id.clone(),
            payload.intent.clone(),
            embedding,
            payload.experience.clone(),
            payload.outcome.clone(),
        );
        Ok(Some(ConsolidatedEpisode {
            id,
            episode,
            reward,
        }))
    }
}

fn consolidation_reward(outcome: &str) -> f32 {
    let lowered = outcome.to_lowercase();
    if lowered.contains("error") || lowered.contains("failed") {
        return 0.0;
    }
    1.0
}
