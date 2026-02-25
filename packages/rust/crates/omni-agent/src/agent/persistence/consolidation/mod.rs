mod payload;

use anyhow::Result;
use omni_memory::{Episode, EpisodeStore};
use std::sync::Arc;
use std::time::Instant;

use crate::observability::SessionEvent;

use super::super::Agent;
use super::persist_memory_state;

struct ConsolidationSummaryPayload {
    intent: String,
    experience: String,
    outcome: String,
    drained_slots: usize,
    drained_turns: usize,
    drained_tool_calls: u32,
}

struct ConsolidatedEpisode {
    id: String,
    episode: Episode,
    reward: f32,
}

impl Agent {
    /// When window >= `consolidation_threshold_turns` and memory is enabled, drain oldest
    /// segment and store as episode.
    pub(super) async fn try_consolidate(&self, session_id: &str) -> Result<()> {
        let (store, threshold, take, consolidate_async) = match (
            self.memory_store.clone(),
            self.config.consolidation_threshold_turns,
            self.config.consolidation_take_turns,
        ) {
            (Some(s), Some(t), take) if take > 0 => (s, t, take, self.config.consolidation_async),
            _ => return Ok(()),
        };
        let Some(ref bounded) = self.bounded_session else {
            return Ok(());
        };
        let started = Instant::now();
        let Some((turn_count, _total_tool_calls, _len)) = bounded.get_stats(session_id).await?
        else {
            return Ok(());
        };
        let turn_count = usize::try_from(turn_count).unwrap_or(usize::MAX);
        if turn_count < threshold {
            return Ok(());
        }
        let drained = bounded.drain_oldest_turns(session_id, take).await?;
        if drained.is_empty() {
            return Ok(());
        }
        let payload = self
            .build_consolidation_payload(bounded, session_id, drained)
            .await?;
        let Some(consolidated) = self
            .build_consolidated_episode(session_id, store.as_ref(), &payload)
            .await?
        else {
            return Ok(());
        };

        self.persist_consolidated_episode(
            session_id,
            store,
            consolidated,
            &payload,
            consolidate_async,
        )
        .await;
        tracing::debug!(
            session_id,
            threshold,
            drained_turns = payload.drained_turns,
            drained_slots = payload.drained_slots,
            drained_tool_calls = payload.drained_tool_calls,
            consolidate_async,
            duration_ms = started.elapsed().as_millis(),
            "session consolidation completed"
        );
        Ok(())
    }

    async fn persist_consolidated_episode(
        &self,
        session_id: &str,
        store: Arc<EpisodeStore>,
        consolidated: ConsolidatedEpisode,
        payload: &ConsolidationSummaryPayload,
        consolidate_async: bool,
    ) {
        let ConsolidatedEpisode {
            id,
            episode,
            reward,
        } = consolidated;
        if consolidate_async {
            self.publish_memory_stream_event(vec![
                ("kind".to_string(), "consolidation_enqueued".to_string()),
                ("session_id".to_string(), session_id.to_string()),
                (
                    "drained_turns".to_string(),
                    payload.drained_turns.to_string(),
                ),
                (
                    "drained_tool_calls".to_string(),
                    payload.drained_tool_calls.to_string(),
                ),
                ("episode_id".to_string(), id.clone()),
            ])
            .await;
            self.spawn_async_consolidation_store_task(session_id, store, id, episode, reward);
            return;
        }

        self.persist_consolidated_episode_sync(session_id, store, id, episode, reward, payload)
            .await;
    }

    fn spawn_async_consolidation_store_task(
        &self,
        session_id: &str,
        store: Arc<EpisodeStore>,
        id: String,
        episode: Episode,
        reward: f32,
    ) {
        let session_id_for_task = session_id.to_string();
        let backend_for_task = self.memory_state_backend.clone();
        tokio::task::spawn_blocking(move || {
            match store.store_for_scope(&session_id_for_task, episode) {
                Ok(_) => {
                    store.update_q(&id, reward);
                    persist_memory_state(
                        backend_for_task.as_ref(),
                        &store,
                        &session_id_for_task,
                        "consolidation",
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        event = SessionEvent::MemoryConsolidationStoreFailed.as_str(),
                        session_id = %session_id_for_task,
                        error = %error,
                        "failed to store consolidated memory episode"
                    );
                }
            }
        });
    }

    async fn persist_consolidated_episode_sync(
        &self,
        session_id: &str,
        store: Arc<EpisodeStore>,
        id: String,
        episode: Episode,
        reward: f32,
        payload: &ConsolidationSummaryPayload,
    ) {
        match store.store_for_scope(session_id, episode) {
            Ok(_) => {
                store.update_q(&id, reward);
                persist_memory_state(
                    self.memory_state_backend.as_ref(),
                    &store,
                    session_id,
                    "consolidation",
                );
                self.publish_memory_stream_event(vec![
                    ("kind".to_string(), "consolidation_stored".to_string()),
                    ("session_id".to_string(), session_id.to_string()),
                    ("episode_id".to_string(), id),
                    ("reward".to_string(), format!("{reward:.3}")),
                    (
                        "drained_turns".to_string(),
                        payload.drained_turns.to_string(),
                    ),
                    (
                        "drained_tool_calls".to_string(),
                        payload.drained_tool_calls.to_string(),
                    ),
                ])
                .await;
            }
            Err(error) => {
                tracing::warn!(
                    event = SessionEvent::MemoryConsolidationStoreFailed.as_str(),
                    session_id,
                    error = %error,
                    "failed to store consolidated memory episode"
                );
                self.publish_memory_stream_event(vec![
                    ("kind".to_string(), "consolidation_store_failed".to_string()),
                    ("session_id".to_string(), session_id.to_string()),
                    ("error".to_string(), error.to_string()),
                ])
                .await;
            }
        }
    }
}
