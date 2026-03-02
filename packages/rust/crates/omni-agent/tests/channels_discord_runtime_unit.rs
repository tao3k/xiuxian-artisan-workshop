//! Top-level integration harness for `channels::discord::runtime` unit lanes.

mod config {
    pub(crate) use omni_agent::{
        AgentConfig, DiscordSettings, RuntimeSettings, load_runtime_settings,
        runtime_settings_paths,
    };
}

mod agent {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::{Result, anyhow, bail};
    use tokio::sync::RwLock;

    use crate::config::AgentConfig;

    pub(crate) use omni_agent::{
        MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot,
        SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot, SessionContextMode,
        SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
        SessionMemoryRecallSnapshot,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub(crate) struct DownstreamAdmissionMetricsSnapshot {
        pub(crate) total: u64,
        pub(crate) admitted: u64,
        pub(crate) rejected: u64,
        pub(crate) rejected_llm_saturated: u64,
        pub(crate) rejected_embedding_saturated: u64,
        pub(crate) reject_rate_pct: u8,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct DownstreamAdmissionRuntimeSnapshot {
        pub(crate) enabled: bool,
        pub(crate) llm_reject_threshold_pct: u8,
        pub(crate) embedding_reject_threshold_pct: u8,
        pub(crate) metrics: DownstreamAdmissionMetricsSnapshot,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub(crate) struct MemoryRuntimeStatusSnapshot {
        pub(crate) enabled: bool,
        pub(crate) configured_backend: Option<String>,
        pub(crate) active_backend: Option<&'static str>,
        pub(crate) strict_startup: Option<bool>,
        pub(crate) startup_load_status: &'static str,
        pub(crate) store_path: Option<String>,
        pub(crate) table_name: Option<String>,
        pub(crate) gate_promote_threshold: Option<f32>,
        pub(crate) gate_obsolete_threshold: Option<f32>,
        pub(crate) gate_promote_min_usage: Option<u32>,
        pub(crate) gate_obsolete_min_usage: Option<u32>,
        pub(crate) gate_promote_failure_rate_ceiling: Option<f32>,
        pub(crate) gate_obsolete_failure_rate_floor: Option<f32>,
        pub(crate) gate_promote_min_ttl_score: Option<f32>,
        pub(crate) gate_obsolete_max_ttl_score: Option<f32>,
        pub(crate) episodes_total: Option<usize>,
        pub(crate) q_values_total: Option<usize>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum SessionRecallFeedbackDirection {
        Up,
        Down,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) struct SessionRecallFeedbackUpdate {
        pub(crate) previous_bias: f32,
        pub(crate) updated_bias: f32,
        pub(crate) direction: SessionRecallFeedbackDirection,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct SessionSystemPromptInjectionSnapshot {
        pub(crate) xml: String,
        pub(crate) qa_count: usize,
        pub(crate) updated_at_unix_ms: u64,
    }

    pub(crate) struct Agent {
        _config: AgentConfig,
        session_messages: Arc<RwLock<HashMap<String, usize>>>,
        backups: Arc<RwLock<HashMap<String, SessionContextSnapshotInfo>>>,
        context_budgets: Arc<RwLock<HashMap<String, SessionContextBudgetSnapshot>>>,
        recall_feedback_bias: Arc<RwLock<HashMap<String, f32>>>,
        injections: Arc<RwLock<HashMap<String, SessionSystemPromptInjectionSnapshot>>>,
    }

    impl Agent {
        pub(crate) async fn from_config(config: AgentConfig) -> Result<Self> {
            Ok(Self {
                _config: config,
                session_messages: Arc::new(RwLock::new(HashMap::new())),
                backups: Arc::new(RwLock::new(HashMap::new())),
                context_budgets: Arc::new(RwLock::new(HashMap::new())),
                recall_feedback_bias: Arc::new(RwLock::new(HashMap::new())),
                injections: Arc::new(RwLock::new(HashMap::new())),
            })
        }

        pub(crate) async fn run_turn(
            &self,
            session_id: &str,
            user_message: &str,
        ) -> Result<String> {
            self.append_turn_for_session(session_id, user_message, "ok")
                .await?;
            Ok("ok".to_string())
        }

        pub(crate) async fn append_turn_for_session(
            &self,
            session_id: &str,
            _user_msg: &str,
            _assistant_msg: &str,
        ) -> Result<()> {
            let mut guard = self.session_messages.write().await;
            let entry = guard.entry(session_id.to_string()).or_insert(0);
            *entry = entry.saturating_add(2);
            Ok(())
        }

        pub(crate) async fn inspect_context_window(
            &self,
            session_id: &str,
        ) -> Result<SessionContextWindowInfo> {
            let messages = self
                .session_messages
                .read()
                .await
                .get(session_id)
                .copied()
                .unwrap_or(0);
            Ok(SessionContextWindowInfo {
                mode: SessionContextMode::Unbounded,
                messages,
                summary_segments: 0,
                window_turns: None,
                window_slots: None,
                total_tool_calls: None,
            })
        }

        pub(crate) async fn reset_context_window(
            &self,
            session_id: &str,
        ) -> Result<SessionContextStats> {
            let cleared_messages = {
                let mut guard = self.session_messages.write().await;
                guard.remove(session_id).unwrap_or(0)
            };
            if cleared_messages > 0 {
                self.backups.write().await.insert(
                    session_id.to_string(),
                    SessionContextSnapshotInfo {
                        messages: cleared_messages,
                        summary_segments: 0,
                        saved_at_unix_ms: Some(now_unix_ms()),
                        saved_age_secs: Some(0),
                    },
                );
            }
            Ok(SessionContextStats {
                messages: cleared_messages,
                summary_segments: 0,
            })
        }

        pub(crate) async fn resume_context_window(
            &self,
            session_id: &str,
        ) -> Result<Option<SessionContextStats>> {
            let Some(snapshot) = self.backups.write().await.remove(session_id) else {
                return Ok(None);
            };
            self.session_messages
                .write()
                .await
                .insert(session_id.to_string(), snapshot.messages);
            Ok(Some(SessionContextStats {
                messages: snapshot.messages,
                summary_segments: snapshot.summary_segments,
            }))
        }

        pub(crate) async fn peek_context_window_backup(
            &self,
            session_id: &str,
        ) -> Result<Option<SessionContextSnapshotInfo>> {
            Ok(self.backups.read().await.get(session_id).copied())
        }

        pub(crate) async fn drop_context_window_backup(&self, session_id: &str) -> Result<bool> {
            Ok(self.backups.write().await.remove(session_id).is_some())
        }

        pub(crate) async fn inspect_context_budget_snapshot(
            &self,
            session_id: &str,
        ) -> Option<SessionContextBudgetSnapshot> {
            self.context_budgets.read().await.get(session_id).copied()
        }

        pub(crate) fn inspect_memory_runtime_status(&self) -> MemoryRuntimeStatusSnapshot {
            MemoryRuntimeStatusSnapshot {
                enabled: false,
                configured_backend: None,
                active_backend: None,
                strict_startup: None,
                startup_load_status: "not_configured",
                store_path: None,
                table_name: None,
                gate_promote_threshold: None,
                gate_obsolete_threshold: None,
                gate_promote_min_usage: None,
                gate_obsolete_min_usage: None,
                gate_promote_failure_rate_ceiling: None,
                gate_obsolete_failure_rate_floor: None,
                gate_promote_min_ttl_score: None,
                gate_obsolete_max_ttl_score: None,
                episodes_total: None,
                q_values_total: None,
            }
        }

        pub(crate) fn downstream_admission_runtime_snapshot(
            &self,
        ) -> DownstreamAdmissionRuntimeSnapshot {
            DownstreamAdmissionRuntimeSnapshot {
                enabled: true,
                llm_reject_threshold_pct: 95,
                embedding_reject_threshold_pct: 95,
                metrics: DownstreamAdmissionMetricsSnapshot::default(),
            }
        }

        pub(crate) async fn inspect_memory_recall_metrics(&self) -> MemoryRecallMetricsSnapshot {
            MemoryRecallMetricsSnapshot {
                captured_at_unix_ms: now_unix_ms(),
                planned_total: 0,
                injected_total: 0,
                skipped_total: 0,
                completed_total: 0,
                selected_total: 0,
                injected_items_total: 0,
                context_chars_injected_total: 0,
                pipeline_duration_ms_total: 0,
                avg_pipeline_duration_ms: 0.0,
                avg_selected_per_completed: 0.0,
                avg_injected_per_injected: 0.0,
                injected_rate: 0.0,
                latency_buckets: MemoryRecallLatencyBucketsSnapshot::default(),
                embedding_success_total: 0,
                embedding_timeout_total: 0,
                embedding_cooldown_reject_total: 0,
                embedding_unavailable_total: 0,
            }
        }

        pub(crate) async fn inspect_memory_recall_snapshot(
            &self,
            _session_id: &str,
        ) -> Option<SessionMemoryRecallSnapshot> {
            None
        }

        pub(crate) fn apply_session_recall_feedback(
            &self,
            session_id: &str,
            direction: SessionRecallFeedbackDirection,
        ) -> Option<SessionRecallFeedbackUpdate> {
            let mut guard = self.recall_feedback_bias.blocking_write();
            let previous_bias = guard.get(session_id).copied().unwrap_or(0.0);
            let delta = match direction {
                SessionRecallFeedbackDirection::Up => 0.1,
                SessionRecallFeedbackDirection::Down => -0.1,
            };
            let updated_bias = previous_bias + delta;
            guard.insert(session_id.to_string(), updated_bias);
            Some(SessionRecallFeedbackUpdate {
                previous_bias,
                updated_bias,
                direction,
            })
        }

        pub(crate) async fn inspect_session_system_prompt_injection(
            &self,
            session_id: &str,
        ) -> Option<SessionSystemPromptInjectionSnapshot> {
            self.injections.read().await.get(session_id).cloned()
        }

        pub(crate) async fn clear_session_system_prompt_injection(
            &self,
            session_id: &str,
        ) -> Result<bool> {
            Ok(self.injections.write().await.remove(session_id).is_some())
        }

        pub(crate) async fn upsert_session_system_prompt_injection_xml(
            &self,
            session_id: &str,
            payload: &str,
        ) -> Result<SessionSystemPromptInjectionSnapshot> {
            let xml = payload.trim();
            if xml.is_empty() {
                bail!("empty injection payload");
            }
            if !xml.contains("<qa>") {
                return Err(anyhow!("expected at least one <qa> element"));
            }
            let snapshot = SessionSystemPromptInjectionSnapshot {
                xml: xml.to_string(),
                qa_count: xml.matches("<qa>").count(),
                updated_at_unix_ms: now_unix_ms(),
            };
            self.injections
                .write()
                .await
                .insert(session_id.to_string(), snapshot.clone());
            Ok(snapshot)
        }
    }

    fn now_unix_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .unwrap_or(0)
    }
}

mod jobs {
    pub(crate) use omni_agent::{
        JobCompletion, JobCompletionKind, JobManager, JobManagerConfig, JobMetricsSnapshot,
        JobStatusSnapshot, TurnRunner,
    };
}

#[async_trait::async_trait]
impl jobs::TurnRunner for agent::Agent {
    async fn run_turn(&self, session_id: &str, user_message: &str) -> anyhow::Result<String> {
        self.run_turn(session_id, user_message).await
    }
}

mod channels {
    pub(crate) mod traits {
        include!("../src/channels/traits.rs");
    }

    pub(crate) mod runtime_snapshot {
        include!("../src/channels/runtime_snapshot.rs");
    }

    pub(crate) mod managed_commands {
        include!("../src/channels/managed_commands/mod.rs");
    }

    pub(crate) mod managed_runtime {
        include!("../src/channels/managed_runtime/mod.rs");
    }

    pub(crate) mod telegram {
        pub(crate) mod runtime {
            pub(crate) mod jobs {
                pub(crate) mod observability {
                    pub(crate) mod json_summary {
                        include!(
                            "../src/channels/telegram/runtime/jobs/observability/json_summary.rs"
                        );
                    }
                }
            }
        }
    }

    pub(crate) mod discord {
        pub(crate) mod runtime {
            mod foreground {
                use crate::channels::managed_runtime::ForegroundQueueMode;

                pub(super) struct DiscordForegroundSnapshot {
                    pub(super) max_in_flight_messages: usize,
                    pub(super) available_permits: usize,
                    pub(super) in_flight_messages: usize,
                    pub(super) task_count: usize,
                    pub(super) queue_mode: ForegroundQueueMode,
                }
            }

            pub(crate) mod dispatch {
                include!("../src/channels/discord/runtime/dispatch/mod.rs");
            }

            mod interrupt {
                include!("../src/channels/discord/runtime/interrupt.rs");
            }

            mod managed {
                include!("../src/channels/discord/runtime/managed/mod.rs");
            }

            mod telemetry {
                include!("../src/channels/discord/runtime/telemetry.rs");
            }

            pub(in crate::channels::discord::runtime) use interrupt::ForegroundInterruptController;

            mod tests {
                include!("discord_runtime/mod.rs");
            }
        }
    }

    pub(crate) use managed_runtime::ForegroundQueueMode;
}
