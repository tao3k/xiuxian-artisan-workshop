use anyhow::{Context, Result};
use omni_window::TurnSlot;

use crate::observability::SessionEvent;

use super::RedisSessionBackend;
use super::backend::{usize_to_i64_saturating, usize_to_u64_saturating};

const APPEND_WINDOW_SLOTS_SCRIPT: &str = r#"
local window_key = KEYS[1]
local tool_key = KEYS[2]
local max_slots = tonumber(ARGV[1])
local ttl = tonumber(ARGV[2])
local appended_tool_calls = tonumber(ARGV[3]) or 0
local payload_count = tonumber(ARGV[4]) or 0

if max_slots < 1 then
  max_slots = 1
end

local current_len = redis.call("LLEN", window_key)
local overflow = current_len + payload_count - max_slots
local removed_tool_calls = 0

if overflow > 0 then
  local removed_payloads = redis.call("LRANGE", window_key, 0, overflow - 1)
  for _, payload in ipairs(removed_payloads) do
    local ok, decoded = pcall(cjson.decode, payload)
    if ok and type(decoded) == "table" then
      local tool_count = tonumber(decoded["tool_count"])
      if tool_count and tool_count > 0 then
        removed_tool_calls = removed_tool_calls + tool_count
      end
    end
  end
end

if payload_count > 0 then
  local args = { window_key }
  for i = 1, payload_count do
    args[#args + 1] = ARGV[4 + i]
  end
  redis.call("RPUSH", unpack(args))
end

redis.call("LTRIM", window_key, -max_slots, -1)

local current_tool_calls = tonumber(redis.call("GET", tool_key) or "0")
local next_tool_calls = current_tool_calls + appended_tool_calls - removed_tool_calls
if next_tool_calls < 0 then
  next_tool_calls = 0
end
redis.call("SET", tool_key, tostring(next_tool_calls))

if ttl > 0 then
  redis.call("EXPIRE", window_key, ttl)
  redis.call("EXPIRE", tool_key, ttl)
end

return next_tool_calls
"#;

const DRAIN_OLDEST_WINDOW_SLOTS_SCRIPT: &str = r#"
local window_key = KEYS[1]
local tool_key = KEYS[2]
local n = tonumber(ARGV[1]) or 0
local ttl = tonumber(ARGV[2]) or 0
if n <= 0 then
  return {}
end

local removed = redis.call("LPOP", window_key, n)
if removed == false or removed == nil then
  return {}
end

local removed_tool_calls = 0
for _, payload in ipairs(removed) do
  local ok, decoded = pcall(cjson.decode, payload)
  if ok and type(decoded) == "table" then
    local tool_count = tonumber(decoded["tool_count"])
    if tool_count and tool_count > 0 then
      removed_tool_calls = removed_tool_calls + tool_count
    end
  end
end

local current_tool_calls = tonumber(redis.call("GET", tool_key) or "0")
local next_tool_calls = current_tool_calls - removed_tool_calls
if next_tool_calls <= 0 then
  redis.call("DEL", tool_key)
else
  redis.call("SET", tool_key, tostring(next_tool_calls))
  if ttl > 0 then
    redis.call("EXPIRE", tool_key, ttl)
  end
end

return removed
"#;

impl RedisSessionBackend {
    fn sum_tool_calls_from_payloads(payloads: &[String]) -> u64 {
        payloads
            .iter()
            .filter_map(|payload| serde_json::from_str::<TurnSlot>(payload).ok())
            .fold(0_u64, |sum, slot| {
                sum.saturating_add(u64::from(slot.tool_count))
            })
    }

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
        let tool_key = self.stream_metrics_session_key("window_tool_calls", session_id);
        let ttl_secs = self.ttl_secs;
        let appended_tool_calls = slots.iter().fold(0_i64, |sum, slot| {
            sum.saturating_add(i64::from(slot.tool_count))
        });

        let total_tool_calls = self
            .run_command::<i64, _>("append_window_slots", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(APPEND_WINDOW_SLOTS_SCRIPT)
                    .arg(2)
                    .arg(&key)
                    .arg(&tool_key)
                    .arg(max_slots_i64)
                    .arg(ttl_secs.unwrap_or(0))
                    .arg(appended_tool_calls)
                    .arg(encoded.len());
                for payload in &encoded {
                    cmd.arg(payload);
                }
                cmd
            })
            .await?;

        tracing::debug!(
            event = SessionEvent::SessionWindowSlotsAppended.as_str(),
            session_id,
            appended_slots = encoded.len(),
            appended_tool_calls,
            total_tool_calls = total_tool_calls.max(0),
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
        let tool_key = self.stream_metrics_session_key("window_tool_calls", session_id);
        let len = self
            .run_command::<usize, _>("get_window_stats_len", || {
                let mut cmd = redis::cmd("LLEN");
                cmd.arg(&key);
                cmd
            })
            .await?;
        if len == 0 {
            let _ = self
                .run_command::<i64, _>("clear_window_tool_calls_when_empty", || {
                    let mut cmd = redis::cmd("DEL");
                    cmd.arg(&tool_key);
                    cmd
                })
                .await;
            return Ok(None);
        }

        let cached_total_tool_calls = self
            .run_command::<Option<u64>, _>("get_window_stats_tool_calls", || {
                let mut cmd = redis::cmd("GET");
                cmd.arg(&tool_key);
                cmd
            })
            .await?;

        let total_tool_calls = if let Some(total_tool_calls) = cached_total_tool_calls {
            total_tool_calls
        } else {
            // Backfill only for pre-migration sessions that do not yet have the O(1) counter key.
            let payloads = self
                .run_command::<Vec<String>, _>("get_window_stats_payload_backfill", || {
                    let mut cmd = redis::cmd("LRANGE");
                    cmd.arg(&key).arg(0).arg(-1);
                    cmd
                })
                .await?;
            let total_tool_calls = Self::sum_tool_calls_from_payloads(&payloads);
            let _ = self
                .run_command::<(), _>("set_window_stats_tool_calls_backfill", || {
                    let mut cmd = redis::cmd("SET");
                    cmd.arg(&tool_key).arg(total_tool_calls);
                    cmd
                })
                .await;
            if let Some(ttl) = self.ttl_secs {
                let _ = self
                    .run_command::<i64, _>("expire_window_stats_tool_calls_backfill", || {
                        let mut cmd = redis::cmd("EXPIRE");
                        cmd.arg(&tool_key).arg(ttl);
                        cmd
                    })
                    .await;
            }
            total_tool_calls
        };

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
        let tool_key = self.stream_metrics_session_key("window_tool_calls", session_id);
        let _ = self
            .run_command::<i64, _>("clear_window", || {
                let mut cmd = redis::cmd("DEL");
                cmd.arg(&key).arg(&tool_key);
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
        let tool_key = self.stream_metrics_session_key("window_tool_calls", session_id);
        let drained_payloads = self
            .run_command::<Vec<String>, _>("drain_oldest_window_slots", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(DRAIN_OLDEST_WINDOW_SLOTS_SCRIPT)
                    .arg(2)
                    .arg(&key)
                    .arg(&tool_key)
                    .arg(usize_to_i64_saturating(n))
                    .arg(self.ttl_secs.unwrap_or(0));
                cmd
            })
            .await?;
        let mut drained = Vec::new();
        for payload in drained_payloads {
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
        let drained_tool_calls = drained.iter().fold(0_u64, |sum, slot| {
            sum.saturating_add(u64::from(slot.tool_count))
        });
        tracing::debug!(
            event = SessionEvent::SessionWindowStatsLoaded.as_str(),
            session_id,
            drained_tool_calls,
            "valkey session window tool-call counter updated after drain"
        );
        Ok(drained)
    }
}
