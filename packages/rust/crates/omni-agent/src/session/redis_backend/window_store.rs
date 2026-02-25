use anyhow::{Context, Result};
use omni_window::TurnSlot;

use crate::observability::SessionEvent;

use super::RedisSessionBackend;
use super::backend::{usize_to_i64_saturating, usize_to_u64_saturating};

impl RedisSessionBackend {
    pub(crate) async fn append_window_slots(
        &self,
        session_id: &str,
        max_slots: usize,
        slots: &[TurnSlot],
    ) -> Result<()> {
        if slots.is_empty() {
            return Ok(());
        }
        let key = self.window_key(session_id);
        let encoded: Vec<String> = slots
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to encode window slots for redis")?;
        let max_slots_i64 = usize_to_i64_saturating(max_slots.max(1));
        let ttl_secs = self.ttl_secs;

        self.run_pipeline::<(), _>("append_window_slots", || {
            let mut pipe = redis::pipe();
            pipe.atomic();
            pipe.cmd("RPUSH").arg(&key);
            for payload in &encoded {
                pipe.arg(payload);
            }
            pipe.ignore();
            pipe.cmd("LTRIM")
                .arg(&key)
                .arg(-max_slots_i64)
                .arg(-1)
                .ignore();
            if let Some(ttl) = ttl_secs {
                pipe.cmd("EXPIRE").arg(&key).arg(ttl).ignore();
            }
            pipe
        })
        .await?;
        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsAppended.as_str(),
            session_id,
            appended_slots = encoded.len(),
            max_slots = max_slots_i64,
            ttl_secs = ?ttl_secs,
            "valkey session window slots appended"
        );
        Ok(())
    }

    pub(crate) async fn get_recent_window_slots(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<TurnSlot>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let key = self.window_key(session_id);
        let limit_i64 = usize_to_i64_saturating(limit);
        let payloads = self
            .run_command::<Vec<String>, _>("get_recent_window_slots", || {
                let mut cmd = redis::cmd("LRANGE");
                cmd.arg(&key).arg(-limit_i64).arg(-1);
                cmd
            })
            .await?;
        let mut out = Vec::with_capacity(payloads.len());
        let mut invalid_payloads = 0usize;
        for payload in payloads {
            match serde_json::from_str::<TurnSlot>(&payload) {
                Ok(slot) => out.push(slot),
                Err(error) => {
                    invalid_payloads += 1;
                    tracing::warn!(
                        event = SessionEvent::SessionWindowSlotsLoaded.as_str(),
                        session_id,
                        error = %error,
                        "invalid turn slot payload in redis session window"
                    );
                }
            }
        }
        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsLoaded.as_str(),
            session_id,
            requested_limit = limit,
            loaded_slots = out.len(),
            invalid_payloads,
            "valkey session window slots loaded"
        );
        Ok(out)
    }

    pub(crate) async fn get_window_stats(
        &self,
        session_id: &str,
    ) -> Result<Option<(u64, u64, usize)>> {
        let key = self.window_key(session_id);
        let len = self
            .run_command::<usize, _>("get_window_stats_len", || {
                let mut cmd = redis::cmd("LLEN");
                cmd.arg(&key);
                cmd
            })
            .await?;
        if len == 0 {
            return Ok(None);
        }
        let payloads = self
            .run_command::<Vec<String>, _>("get_window_stats_payload", || {
                let mut cmd = redis::cmd("LRANGE");
                cmd.arg(&key).arg(0).arg(-1);
                cmd
            })
            .await?;
        let mut total_tool_calls: u64 = 0;
        for payload in payloads {
            if let Ok(slot) = serde_json::from_str::<TurnSlot>(&payload) {
                total_tool_calls = total_tool_calls.saturating_add(u64::from(slot.tool_count));
            }
        }
        tracing::debug!(
            event = SessionEvent::SessionWindowStatsLoaded.as_str(),
            session_id,
            slots = len,
            total_tool_calls,
            "valkey session window stats loaded"
        );
        Ok(Some((usize_to_u64_saturating(len), total_tool_calls, len)))
    }

    pub(crate) async fn clear_window(&self, session_id: &str) -> Result<()> {
        let key = self.window_key(session_id);
        let _ = self
            .run_command::<i64, _>("clear_window", || {
                let mut cmd = redis::cmd("DEL");
                cmd.arg(&key);
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionWindowCleared.as_str(),
            session_id,
            "valkey session window cleared"
        );
        Ok(())
    }

    pub(crate) async fn drain_oldest_window_slots(
        &self,
        session_id: &str,
        n: usize,
    ) -> Result<Vec<TurnSlot>> {
        if n == 0 {
            return Ok(Vec::new());
        }
        let key = self.window_key(session_id);
        let mut drained = Vec::new();
        for _ in 0..n {
            let popped = self
                .run_command::<Option<String>, _>("drain_oldest_window_slots", || {
                    let mut cmd = redis::cmd("LPOP");
                    cmd.arg(&key);
                    cmd
                })
                .await?;
            let Some(payload) = popped else {
                break;
            };
            match serde_json::from_str::<TurnSlot>(&payload) {
                Ok(slot) => drained.push(slot),
                Err(error) => {
                    tracing::warn!(
                        event = SessionEvent::SessionWindowSlotsDrained.as_str(),
                        session_id,
                        error = %error,
                        "invalid drained turn slot payload from redis"
                    );
                }
            }
        }
        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsDrained.as_str(),
            session_id,
            requested_slots = n,
            drained_slots = drained.len(),
            "valkey session window slots drained"
        );
        Ok(drained)
    }
}
