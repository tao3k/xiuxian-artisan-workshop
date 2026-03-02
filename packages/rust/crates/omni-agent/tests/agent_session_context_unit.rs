//! Top-level integration harness for `agent::session_context`.

mod config {
    pub(crate) use omni_agent::AgentConfig;
}

mod observability {
    #[derive(Clone, Copy, Debug)]
    pub(crate) enum SessionEvent {
        BoundedStatsLoaded,
        ContextBackupCaptured,
        ContextWindowReset,
        ContextWindowResumeMissing,
        ContextWindowResumed,
        ContextWindowSnapshotDropped,
        ContextWindowSnapshotInspected,
        SessionMessagesLoaded,
    }

    impl SessionEvent {
        pub(crate) const fn as_str(self) -> &'static str {
            match self {
                Self::BoundedStatsLoaded => "bounded_stats_loaded",
                Self::ContextBackupCaptured => "context_backup_captured",
                Self::ContextWindowReset => "context_window_reset",
                Self::ContextWindowResumeMissing => "context_window_resume_missing",
                Self::ContextWindowResumed => "context_window_resumed",
                Self::ContextWindowSnapshotDropped => "context_window_snapshot_dropped",
                Self::ContextWindowSnapshotInspected => "context_window_snapshot_inspected",
                Self::SessionMessagesLoaded => "session_messages_loaded",
            }
        }
    }
}

mod session {
    pub(crate) use omni_agent::{
        BoundedSessionStore, ChatMessage, SessionStore, SessionSummarySegment,
    };
}

mod agent {
    use std::collections::HashMap;
    use std::sync::Arc;

    use anyhow::Result;
    use tokio::sync::RwLock;

    use crate::config::AgentConfig;
    use crate::session::{BoundedSessionStore, ChatMessage, SessionStore};

    pub(crate) struct Agent {
        pub(crate) config: AgentConfig,
        pub(crate) session: SessionStore,
        pub(crate) session_reset_idle_timeout_ms: Option<u64>,
        pub(crate) session_last_activity_unix_ms: Arc<RwLock<HashMap<String, u64>>>,
        pub(crate) bounded_session: Option<BoundedSessionStore>,
    }

    impl Agent {
        pub(crate) async fn from_config(config: AgentConfig) -> Result<Self> {
            let session = SessionStore::new()?;
            let bounded_session = match config.window_max_turns {
                Some(max_turns) => Some(BoundedSessionStore::new_with_limits(
                    max_turns,
                    config.summary_max_segments,
                    config.summary_max_chars,
                )?),
                None => None,
            };
            Ok(Self {
                config,
                session,
                session_reset_idle_timeout_ms: None,
                session_last_activity_unix_ms: Arc::new(RwLock::new(HashMap::new())),
                bounded_session,
            })
        }

        pub(crate) async fn clear_session(&self, session_id: &str) -> Result<()> {
            if let Some(ref bounded_session) = self.bounded_session {
                bounded_session.clear(session_id).await?;
            }
            self.session.clear(session_id).await
        }

        async fn append_turn_to_session(
            &self,
            session_id: &str,
            user_msg: &str,
            assistant_msg: &str,
            tool_count: u32,
        ) -> Result<()> {
            if let Some(ref bounded_session) = self.bounded_session {
                bounded_session
                    .append_turn(session_id, user_msg, assistant_msg, tool_count)
                    .await?;
                return Ok(());
            }

            self.session
                .append(
                    session_id,
                    vec![
                        ChatMessage {
                            role: "user".to_string(),
                            content: Some(user_msg.to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                        },
                        ChatMessage {
                            role: "assistant".to_string(),
                            content: Some(assistant_msg.to_string()),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                        },
                    ],
                )
                .await
        }
    }

    pub(crate) mod session_context {
        include!("../src/agent/session_context/mod.rs");

        mod tests {
            include!("unit/agent/session_context_tests.rs");
        }
    }
}
