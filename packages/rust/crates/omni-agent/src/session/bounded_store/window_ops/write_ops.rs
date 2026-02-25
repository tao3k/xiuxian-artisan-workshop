use anyhow::{Context, Result};
use omni_window::{SessionWindow, TurnSlot};

use crate::observability::SessionEvent;

use super::super::BoundedSessionStore;

impl BoundedSessionStore {
    /// Append one user/assistant turn. Creates the session window if missing.
    ///
    /// # Errors
    /// Returns an error when appending bounded window slots to Valkey fails.
    pub async fn append_turn(
        &self,
        session_id: &str,
        user_content: &str,
        assistant_content: &str,
        tool_count: u32,
    ) -> Result<()> {
        let slots = vec![
            TurnSlot::new("user", user_content, 0),
            TurnSlot::new("assistant", assistant_content, tool_count),
        ];
        if let Some(ref redis) = self.redis {
            redis
                .append_window_slots(session_id, self.max_slots, &slots)
                .await
                .with_context(|| {
                    format!("valkey bounded session append failed for session_id={session_id}")
                })?;
            tracing::debug!(
                event = SessionEvent::BoundedTurnAppended.as_str(),
                session_id,
                tool_count,
                user_chars = user_content.chars().count(),
                assistant_chars = assistant_content.chars().count(),
                backend = "valkey",
                "bounded session turn appended"
            );
            return Ok(());
        }

        let mut guard = self.inner.write().await;
        let window = guard
            .entry(session_id.to_string())
            .or_insert_with(|| SessionWindow::new(session_id, self.max_slots));
        window.append_turn("user", user_content, 0, None);
        window.append_turn("assistant", assistant_content, tool_count, None);
        tracing::debug!(
            event = SessionEvent::BoundedTurnAppended.as_str(),
            session_id,
            tool_count,
            user_chars = user_content.chars().count(),
            assistant_chars = assistant_content.chars().count(),
            backend = "memory",
            "bounded session turn appended"
        );
        Ok(())
    }

    /// Replace active window slots for a session with an exact raw snapshot.
    ///
    /// # Errors
    /// Returns an error when replacing bounded window state in Valkey fails.
    pub async fn replace_window_slots(&self, session_id: &str, slots: &[TurnSlot]) -> Result<()> {
        if let Some(ref redis) = self.redis {
            redis.clear_window(session_id).await.with_context(|| {
                format!("valkey bounded session clear failed for session_id={session_id}")
            })?;
            if !slots.is_empty() {
                redis
                    .append_window_slots(session_id, self.max_slots, slots)
                    .await
                    .with_context(|| {
                        format!("valkey bounded session restore failed for session_id={session_id}")
                    })?;
            }
            tracing::debug!(
                event = SessionEvent::SessionWindowSlotsAppended.as_str(),
                session_id,
                replaced_slots = slots.len(),
                backend = "valkey",
                "bounded session window slots replaced"
            );
            return Ok(());
        }

        let mut guard = self.inner.write().await;
        if slots.is_empty() {
            guard.remove(session_id);
            tracing::debug!(
                event = SessionEvent::SessionWindowCleared.as_str(),
                session_id,
                backend = "memory",
                "bounded session window slots replaced with empty snapshot"
            );
            return Ok(());
        }

        let mut window = SessionWindow::new(session_id, self.max_slots);
        for slot in slots {
            window.append_turn(
                &slot.role,
                &slot.content,
                slot.tool_count,
                slot.checkpoint_id.as_deref(),
            );
        }
        guard.insert(session_id.to_string(), window);
        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsAppended.as_str(),
            session_id,
            replaced_slots = slots.len(),
            backend = "memory",
            "bounded session window slots replaced"
        );
        Ok(())
    }

    /// Clear the session (e.g. on explicit clear).
    ///
    /// # Errors
    /// Returns an error when clearing bounded-session state in Valkey fails.
    pub async fn clear(&self, session_id: &str) -> Result<()> {
        if let Some(ref redis) = self.redis {
            redis.clear_window(session_id).await.with_context(|| {
                format!("valkey bounded session clear failed for session_id={session_id}")
            })?;
            redis.clear_summary(session_id).await.with_context(|| {
                format!("valkey bounded summary clear failed for session_id={session_id}")
            })?;
            tracing::debug!(
                event = SessionEvent::BoundedCleared.as_str(),
                session_id,
                backend = "valkey",
                "bounded session cleared"
            );
            return Ok(());
        }
        let mut guard = self.inner.write().await;
        guard.remove(session_id);
        let mut summaries = self.summaries.write().await;
        summaries.remove(session_id);
        tracing::debug!(
            event = SessionEvent::BoundedCleared.as_str(),
            session_id,
            backend = "memory",
            "bounded session cleared"
        );
        Ok(())
    }

    /// Drain the oldest `n` turns for consolidation. Returns (role, content, `tool_count`) per turn.
    /// Call when window is at or above consolidation threshold; then summarise and store as episode.
    ///
    /// # Errors
    /// Returns an error when draining bounded-session slots from Valkey fails.
    pub async fn drain_oldest_turns(
        &self,
        session_id: &str,
        n: usize,
    ) -> Result<Vec<(String, String, u32)>> {
        let n_slots = n.saturating_mul(2);
        if let Some(ref redis) = self.redis {
            let slots = redis
                .drain_oldest_window_slots(session_id, n_slots)
                .await
                .with_context(|| {
                    format!("valkey bounded session drain failed for session_id={session_id}")
                })?;
            let mut guard = self.inner.write().await;
            if let Some(window) = guard.get_mut(session_id) {
                let _ = window.drain_oldest_turns(n_slots);
            }
            let drained = slots
                .into_iter()
                .map(|slot| (slot.role, slot.content, slot.tool_count))
                .collect::<Vec<_>>();
            tracing::debug!(
                event = SessionEvent::BoundedTurnsDrained.as_str(),
                session_id,
                requested_turns = n,
                drained_turns = drained.len() / 2,
                drained_slots = drained.len(),
                backend = "valkey",
                "bounded session oldest turns drained"
            );
            return Ok(drained);
        }

        let mut guard = self.inner.write().await;
        let Some(window) = guard.get_mut(session_id) else {
            return Ok(Vec::new());
        };
        let slots = window.drain_oldest_turns(n_slots);
        let drained = slots
            .into_iter()
            .map(|slot| (slot.role, slot.content, slot.tool_count))
            .collect::<Vec<_>>();
        tracing::debug!(
            event = SessionEvent::BoundedTurnsDrained.as_str(),
            session_id,
            requested_turns = n,
            drained_turns = drained.len() / 2,
            drained_slots = drained.len(),
            backend = "memory",
            "bounded session oldest turns drained"
        );
        Ok(drained)
    }
}
