use crate::observability::SessionEvent;
use crate::session::ChatMessage;
use anyhow::Result;

use super::types::{SessionContextBackup, SessionContextBackupMetadata};
use super::{Agent, SessionContextStats, backup_metadata_session_id, now_unix_ms};

impl Agent {
    pub(super) async fn capture_session_backup(
        &self,
        session_id: &str,
    ) -> Result<SessionContextBackup> {
        if let Some(ref w) = self.bounded_session {
            let limit_slots = self
                .config
                .window_max_turns
                .unwrap_or(512)
                .saturating_mul(2);
            let window_slots = w.get_recent_slots(session_id, limit_slots).await?;
            let summary_segments = w
                .get_recent_summary_segments(session_id, self.config.summary_max_segments)
                .await?;
            tracing::debug!(
                event = SessionEvent::ContextBackupCaptured.as_str(),
                session_id,
                messages = window_slots.len(),
                summary_segments = summary_segments.len(),
                backend = "bounded",
                "session context backup captured"
            );
            return Ok(SessionContextBackup {
                messages: Vec::new(),
                summary_segments,
                window_slots,
            });
        }

        let messages = self.session.get(session_id).await?;
        tracing::debug!(
            event = SessionEvent::ContextBackupCaptured.as_str(),
            session_id,
            messages = messages.len(),
            backend = "session-store",
            "session context backup captured"
        );
        Ok(SessionContextBackup {
            messages,
            summary_segments: Vec::new(),
            window_slots: Vec::new(),
        })
    }

    pub(super) async fn store_session_backup(
        &self,
        session_id: &str,
        backup: &SessionContextBackup,
    ) -> Result<()> {
        self.clear_session(session_id).await?;

        if let Some(ref w) = self.bounded_session {
            for segment in &backup.summary_segments {
                w.append_summary_segment(session_id, segment.clone())
                    .await?;
            }
            w.replace_window_slots(session_id, &backup.window_slots)
                .await?;
            return Ok(());
        }

        self.session
            .append(session_id, backup.messages.clone())
            .await
    }

    pub(super) async fn restore_session_backup(
        &self,
        session_id: &str,
        backup: SessionContextBackup,
    ) -> Result<()> {
        self.clear_session(session_id).await?;

        if let Some(ref w) = self.bounded_session {
            for segment in backup.summary_segments {
                w.append_summary_segment(session_id, segment).await?;
            }
            w.replace_window_slots(session_id, &backup.window_slots)
                .await?;
            return Ok(());
        }

        self.session.append(session_id, backup.messages).await
    }

    pub(super) async fn store_backup_metadata(
        &self,
        session_id: &str,
        stats: SessionContextStats,
    ) -> Result<()> {
        let metadata_session_id = backup_metadata_session_id(session_id);
        let metadata = SessionContextBackupMetadata {
            messages: stats.messages,
            summary_segments: stats.summary_segments,
            saved_at_unix_ms: now_unix_ms(),
        };
        let content = serde_json::to_string(&metadata)?;
        self.session.clear(&metadata_session_id).await?;
        self.session
            .append(
                &metadata_session_id,
                vec![ChatMessage {
                    role: "system".to_string(),
                    content: Some(content),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }],
            )
            .await
    }

    pub(super) async fn load_backup_metadata(
        &self,
        session_id: &str,
    ) -> Result<Option<SessionContextBackupMetadata>> {
        let metadata_session_id = backup_metadata_session_id(session_id);
        let messages = self.session.get(&metadata_session_id).await?;
        let Some(content) = messages
            .into_iter()
            .rev()
            .find_map(|message| message.content)
        else {
            return Ok(None);
        };
        Ok(serde_json::from_str(&content).ok())
    }

    pub(super) async fn clear_backup_metadata(&self, session_id: &str) -> Result<()> {
        self.session
            .clear(&backup_metadata_session_id(session_id))
            .await
    }
}
