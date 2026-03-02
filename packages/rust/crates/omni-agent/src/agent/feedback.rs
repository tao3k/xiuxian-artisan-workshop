use anyhow::Result;

use super::memory::{RecalledEpisodeCandidate, apply_recall_credit};
use super::memory_recall_feedback::{
    RECALL_FEEDBACK_SOURCE_COMMAND, RecallOutcome, ToolExecutionSummary, resolve_feedback_outcome,
    update_feedback_bias,
};
use super::{Agent, SessionRecallFeedbackDirection, SessionRecallFeedbackUpdate};
use crate::observability::SessionEvent;

impl Agent {
    /// Clear session history for a session.
    ///
    /// # Errors
    /// Returns an error when clearing persisted session history fails.
    pub async fn clear_session(&self, session_id: &str) -> Result<()> {
        if let Some(ref w) = self.bounded_session {
            w.clear(session_id).await?;
        }
        if let Some(store) = self.memory_store.as_ref() {
            let _ = store.clear_recall_feedback_bias_for_scope(session_id);
            self.clear_memory_recall_feedback_bias_atomic(session_id, "session_clear");
        }
        self.reflection_policy_hints
            .write()
            .await
            .remove(session_id);
        let _ = self.clear_session_system_prompt_injection(session_id).await;
        self.session.clear(session_id).await
    }

    /// Apply explicit recall feedback for a session.
    ///
    /// Returns `None` when memory is disabled.
    pub fn apply_session_recall_feedback(
        &self,
        session_id: &str,
        direction: SessionRecallFeedbackDirection,
    ) -> Option<SessionRecallFeedbackUpdate> {
        self.memory_store.as_ref()?;
        let outcome = match direction {
            SessionRecallFeedbackDirection::Up => RecallOutcome::Success,
            SessionRecallFeedbackDirection::Down => RecallOutcome::Failure,
        };
        let (previous, updated) = self.apply_recall_feedback_outcome(
            session_id,
            outcome,
            RECALL_FEEDBACK_SOURCE_COMMAND,
            None,
        );
        Some(SessionRecallFeedbackUpdate {
            previous_bias: previous,
            updated_bias: updated,
            direction,
        })
    }

    pub(in crate::agent) fn recall_feedback_bias(&self, session_id: &str) -> f32 {
        self.memory_store.as_ref().map_or(0.0, |store| {
            store.recall_feedback_bias_for_scope(session_id)
        })
    }

    pub(in crate::agent) fn update_recall_feedback(
        &self,
        session_id: &str,
        user_message: &str,
        assistant_message: &str,
        tool_summary: Option<&ToolExecutionSummary>,
    ) -> Option<RecallOutcome> {
        self.memory_store.as_ref()?;
        let (outcome, source) =
            resolve_feedback_outcome(user_message, tool_summary, assistant_message);
        self.apply_recall_feedback_outcome(session_id, outcome, source, tool_summary);
        Some(outcome)
    }

    pub(in crate::agent) fn apply_memory_recall_credit(
        &self,
        session_id: &str,
        candidates: &[RecalledEpisodeCandidate],
        outcome: Option<RecallOutcome>,
    ) {
        let Some(store) = self.memory_store.as_ref() else {
            return;
        };
        let Some(outcome) = outcome else {
            return;
        };
        if candidates.is_empty() {
            return;
        }
        let updates = apply_recall_credit(store, candidates, outcome);
        if updates.is_empty() {
            return;
        }
        for update in &updates {
            self.persist_memory_q_atomic(
                Some(session_id),
                &update.episode_id,
                update.updated_q,
                "recall_credit",
            );
        }
        let total_delta: f32 = updates.iter().map(|u| u.updated_q - u.previous_q).sum();
        let update_count = u16::try_from(updates.len()).unwrap_or(u16::MAX);
        let avg_weight = updates.iter().map(|u| u.weight).sum::<f32>() / f32::from(update_count);
        tracing::debug!(
            event = SessionEvent::MemoryRecallCreditApplied.as_str(),
            session_id,
            outcome = outcome.as_str(),
            candidates = candidates.len(),
            applied = updates.len(),
            avg_weight,
            total_q_delta = total_delta,
            "memory recall credit applied"
        );
    }

    pub(in crate::agent) fn apply_recall_feedback_outcome(
        &self,
        session_id: &str,
        outcome: RecallOutcome,
        source: &str,
        tool_summary: Option<&ToolExecutionSummary>,
    ) -> (f32, f32) {
        let previous = self.recall_feedback_bias(session_id);
        let updated = update_feedback_bias(previous, outcome);
        if let Some(store) = self.memory_store.as_ref() {
            let _ = store.set_recall_feedback_bias_for_scope(session_id, updated);
            self.persist_memory_recall_feedback_bias_atomic(
                session_id,
                updated,
                "recall_feedback_updated",
            );
        }
        tracing::debug!(
            event = SessionEvent::MemoryRecallFeedbackUpdated.as_str(),
            session_id,
            outcome = outcome.as_str(),
            feedback_source = source,
            tool_attempted = tool_summary.map_or(0, |summary| summary.attempted),
            tool_succeeded = tool_summary.map_or(0, |summary| summary.succeeded),
            tool_failed = tool_summary.map_or(0, |summary| summary.failed),
            recall_feedback_bias_before = previous,
            recall_feedback_bias_after = updated,
            "memory recall feedback updated"
        );
        (previous, updated)
    }
}
