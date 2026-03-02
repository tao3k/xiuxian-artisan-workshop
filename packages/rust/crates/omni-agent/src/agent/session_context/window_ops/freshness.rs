use anyhow::Result;

use crate::observability::SessionEvent;
use crate::session::{ChatMessage, SessionSummarySegment};

use super::super::{Agent, now_unix_ms};

const IDLE_SESSION_RESET_NOTICE: &str = "Previous session expired due to inactivity.";
const SESSION_RESET_NOTICE_NAME: &str = "session_reset_notice";

impl Agent {
    pub(crate) async fn enforce_session_reset_policy(&self, session_id: &str) -> Result<()> {
        let now_ms = now_unix_ms();
        let Some(idle_timeout_ms) = self.session_reset_idle_timeout_ms else {
            self.record_session_activity(session_id, now_ms).await;
            return Ok(());
        };

        let previous_activity_ms = {
            let guard = self.session_last_activity_unix_ms.read().await;
            guard.get(session_id).copied()
        };
        let stale = previous_activity_ms
            .is_some_and(|previous_ms| now_ms.saturating_sub(previous_ms) >= idle_timeout_ms);
        if stale {
            let stats = self.reset_context_window(session_id).await?;
            if stats.messages > 0 || stats.summary_segments > 0 {
                self.inject_idle_reset_notice(session_id, now_ms).await?;
            }
            tracing::info!(
                event = SessionEvent::ContextWindowReset.as_str(),
                session_id,
                reason = "idle_timeout",
                idle_timeout_ms,
                messages_cleared = stats.messages,
                summary_segments_cleared = stats.summary_segments,
                "session context auto-reset due to idle timeout"
            );
        }

        self.record_session_activity(session_id, now_ms).await;
        Ok(())
    }

    async fn record_session_activity(&self, session_id: &str, now_ms: u64) {
        let mut guard = self.session_last_activity_unix_ms.write().await;
        guard.insert(session_id.to_string(), now_ms);
    }

    async fn inject_idle_reset_notice(&self, session_id: &str, now_ms: u64) -> Result<()> {
        if let Some(ref bounded_session) = self.bounded_session {
            let summary =
                SessionSummarySegment::new(IDLE_SESSION_RESET_NOTICE.to_string(), 0, 0, now_ms);
            bounded_session
                .append_summary_segment(session_id, summary)
                .await?;
            return Ok(());
        }

        self.session
            .append(
                session_id,
                vec![ChatMessage {
                    role: "system".to_string(),
                    content: Some(IDLE_SESSION_RESET_NOTICE.to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: Some(SESSION_RESET_NOTICE_NAME.to_string()),
                }],
            )
            .await
    }
}
