use crate::observability::SessionEvent;
use anyhow::Result;

use super::super::{
    Agent, SessionContextSnapshotInfo, SessionContextStats, backup_metadata_session_id,
    backup_session_id, now_unix_ms,
};

impl Agent {
    /// Reset active context and snapshot it under backup keys.
    ///
    /// # Errors
    /// Returns an error when snapshotting, clearing, or backup metadata persistence fails.
    pub async fn reset_context_window(&self, session_id: &str) -> Result<SessionContextStats> {
        let backup_session_id = backup_session_id(session_id);
        let metadata_session_id = backup_metadata_session_id(session_id);

        if let Some(ref w) = self.bounded_session
            && let Some((messages, summary_segments)) = w
                .atomic_reset_snapshot(
                    session_id,
                    &backup_session_id,
                    &metadata_session_id,
                    now_unix_ms(),
                )
                .await?
        {
            let stats = SessionContextStats {
                messages,
                summary_segments,
            };
            tracing::debug!(
                event = SessionEvent::ContextWindowReset.as_str(),
                session_id,
                messages = stats.messages,
                summary_segments = stats.summary_segments,
                backup_saved = stats.messages > 0 || stats.summary_segments > 0,
                mode = "bounded-atomic",
                "session context window reset"
            );
            return Ok(stats);
        }

        let backup = self.capture_session_backup(session_id).await?;
        let stats = backup.stats();
        let backup_was_empty = backup.is_empty();

        self.clear_session(session_id).await?;

        // Keep prior snapshot when current context is already empty.
        if !backup_was_empty {
            self.store_session_backup(&backup_session_id, &backup)
                .await?;
            self.store_backup_metadata(session_id, stats).await?;
        }

        tracing::debug!(
            event = SessionEvent::ContextWindowReset.as_str(),
            session_id,
            messages = stats.messages,
            summary_segments = stats.summary_segments,
            backup_saved = !backup_was_empty,
            "session context window reset"
        );
        Ok(stats)
    }

    /// Restore the latest saved context snapshot after `/reset` or `/clear`.
    ///
    /// Returns `Ok(None)` when no snapshot exists for this session.
    ///
    /// # Errors
    /// Returns an error when loading snapshot payloads or restoring session state fails.
    pub async fn resume_context_window(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionContextStats>> {
        let backup_session_id = backup_session_id(session_id);
        let metadata_session_id = backup_metadata_session_id(session_id);

        if let Some(ref w) = self.bounded_session
            && let Some((messages, summary_segments)) = w
                .atomic_resume_snapshot(session_id, &backup_session_id, &metadata_session_id)
                .await?
        {
            let stats = SessionContextStats {
                messages,
                summary_segments,
            };
            tracing::debug!(
                event = SessionEvent::ContextWindowResumed.as_str(),
                session_id,
                messages = stats.messages,
                summary_segments = stats.summary_segments,
                mode = "bounded-atomic",
                "session context window resumed"
            );
            return Ok(Some(stats));
        }

        let backup = self.capture_session_backup(&backup_session_id).await?;
        if backup.is_empty() {
            tracing::debug!(
                event = SessionEvent::ContextWindowResumeMissing.as_str(),
                session_id,
                "session context resume requested but no snapshot found"
            );
            return Ok(None);
        }

        let stats = backup.stats();
        self.restore_session_backup(session_id, backup).await?;
        self.clear_session(&backup_session_id).await?;
        self.clear_backup_metadata(session_id).await?;
        tracing::debug!(
            event = SessionEvent::ContextWindowResumed.as_str(),
            session_id,
            messages = stats.messages,
            summary_segments = stats.summary_segments,
            "session context window resumed"
        );
        Ok(Some(stats))
    }

    /// Drop saved context snapshot created by `/reset` or `/clear` without restoring it.
    ///
    /// Returns `Ok(true)` when a snapshot existed and was removed.
    ///
    /// # Errors
    /// Returns an error when checking or deleting backup snapshot state fails.
    pub async fn drop_context_window_backup(&self, session_id: &str) -> Result<bool> {
        let backup_session_id = backup_session_id(session_id);
        let metadata_session_id = backup_metadata_session_id(session_id);

        if let Some(ref w) = self.bounded_session
            && let Some(dropped) = w
                .atomic_drop_snapshot(&backup_session_id, &metadata_session_id)
                .await?
        {
            tracing::debug!(
                event = SessionEvent::ContextWindowSnapshotDropped.as_str(),
                session_id,
                dropped,
                mode = "bounded-atomic",
                "session context snapshot dropped"
            );
            return Ok(dropped);
        }

        let has_backup = if let Some(ref w) = self.bounded_session {
            let has_window_slots = w
                .get_stats(&backup_session_id)
                .await?
                .is_some_and(|(_, _, ring_len)| ring_len > 0);
            let summary_segments = w.get_summary_segment_count(&backup_session_id).await?;
            has_window_slots || summary_segments > 0
        } else {
            self.session.len(&backup_session_id).await? > 0
        };

        if has_backup {
            self.clear_session(&backup_session_id).await?;
        }
        self.clear_backup_metadata(session_id).await?;
        tracing::debug!(
            event = SessionEvent::ContextWindowSnapshotDropped.as_str(),
            session_id,
            dropped = has_backup,
            "session context snapshot dropped"
        );
        Ok(has_backup)
    }

    /// Inspect the currently saved backup snapshot metadata and counters.
    ///
    /// # Errors
    /// Returns an error when reading backup payload or metadata fails.
    pub async fn peek_context_window_backup(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionContextSnapshotInfo>> {
        let backup = self
            .capture_session_backup(&backup_session_id(session_id))
            .await?;
        if backup.is_empty() {
            return Ok(None);
        }

        let metadata = self.load_backup_metadata(session_id).await?;
        let (saved_at_unix_ms, saved_age_secs) = metadata.map_or((None, None), |meta| {
            (
                Some(meta.saved_at_unix_ms),
                Some(
                    now_unix_ms()
                        .saturating_sub(meta.saved_at_unix_ms)
                        .saturating_div(1000),
                ),
            )
        });
        let info = SessionContextSnapshotInfo {
            messages: backup.stats().messages,
            summary_segments: backup.stats().summary_segments,
            saved_at_unix_ms,
            saved_age_secs,
        };
        tracing::debug!(
            event = SessionEvent::ContextWindowSnapshotInspected.as_str(),
            session_id,
            messages = info.messages,
            summary_segments = info.summary_segments,
            saved_at_unix_ms = ?info.saved_at_unix_ms,
            saved_age_secs = ?info.saved_age_secs,
            "session context backup snapshot inspected"
        );
        Ok(Some(info))
    }
}
