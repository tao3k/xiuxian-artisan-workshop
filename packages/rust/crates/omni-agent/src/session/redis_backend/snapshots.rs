use anyhow::Result;

use crate::observability::SessionEvent;

use super::RedisSessionBackend;

impl RedisSessionBackend {
    pub(crate) async fn atomic_reset_bounded_snapshot(
        &self,
        session_id: &str,
        backup_session_id: &str,
        metadata_session_id: &str,
        saved_at_unix_ms: u64,
    ) -> Result<(usize, usize)> {
        let src_window = self.window_key(session_id);
        let src_summary = self.summary_key(session_id);
        let src_tool = self.stream_metrics_session_key("window_tool_calls", session_id);
        let dst_window = self.window_key(backup_session_id);
        let dst_summary = self.summary_key(backup_session_id);
        let dst_tool = self.stream_metrics_session_key("window_tool_calls", backup_session_id);
        let metadata_key = self.messages_key(metadata_session_id);
        let ttl_secs = self.ttl_secs.unwrap_or(0);

        let script = r#"
local src_window = KEYS[1]
local src_summary = KEYS[2]
local src_tool = KEYS[3]
local dst_window = KEYS[4]
local dst_summary = KEYS[5]
local dst_tool = KEYS[6]
local metadata_key = KEYS[7]
local saved_at = tonumber(ARGV[1])
local ttl = tonumber(ARGV[2])

redis.call("DEL", dst_window, dst_summary, dst_tool, metadata_key)

local window_len = redis.call("LLEN", src_window)
local summary_len = redis.call("LLEN", src_summary)

if window_len > 0 then
  redis.call("RENAME", src_window, dst_window)
  if redis.call("EXISTS", src_tool) == 1 then
    redis.call("RENAME", src_tool, dst_tool)
  else
    redis.call("SET", dst_tool, "0")
  end
elseif redis.call("EXISTS", src_tool) == 1 then
  redis.call("DEL", src_tool)
end
if summary_len > 0 then
  redis.call("RENAME", src_summary, dst_summary)
end

if window_len > 0 or summary_len > 0 then
  local metadata_payload = cjson.encode({
    messages = window_len,
    summary_segments = summary_len,
    saved_at_unix_ms = saved_at
  })
  local chat_message_payload = cjson.encode({
    role = "system",
    content = metadata_payload
  })
  redis.call("RPUSH", metadata_key, chat_message_payload)
  if ttl > 0 then
    redis.call("EXPIRE", dst_tool, ttl)
    redis.call("EXPIRE", metadata_key, ttl)
  end
end

return {window_len, summary_len}
"#;

        let result = self
            .run_command::<(usize, usize), _>("atomic_reset_bounded_snapshot", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script)
                    .arg(7)
                    .arg(&src_window)
                    .arg(&src_summary)
                    .arg(&src_tool)
                    .arg(&dst_window)
                    .arg(&dst_summary)
                    .arg(&dst_tool)
                    .arg(&metadata_key)
                    .arg(saved_at_unix_ms)
                    .arg(ttl_secs);
                cmd
            })
            .await?;

        tracing::debug!(
            event = SessionEvent::ContextWindowReset.as_str(),
            session_id,
            backup_session_id,
            messages = result.0,
            summary_segments = result.1,
            backend = "valkey",
            "atomic bounded context snapshot reset completed"
        );
        Ok(result)
    }

    pub(crate) async fn atomic_resume_bounded_snapshot(
        &self,
        session_id: &str,
        backup_session_id: &str,
        metadata_session_id: &str,
    ) -> Result<Option<(usize, usize)>> {
        let src_window = self.window_key(backup_session_id);
        let src_summary = self.summary_key(backup_session_id);
        let src_tool = self.stream_metrics_session_key("window_tool_calls", backup_session_id);
        let dst_window = self.window_key(session_id);
        let dst_summary = self.summary_key(session_id);
        let dst_tool = self.stream_metrics_session_key("window_tool_calls", session_id);
        let metadata_key = self.messages_key(metadata_session_id);

        let script = r#"
local src_window = KEYS[1]
local src_summary = KEYS[2]
local src_tool = KEYS[3]
local dst_window = KEYS[4]
local dst_summary = KEYS[5]
local dst_tool = KEYS[6]
local metadata_key = KEYS[7]

local window_len = redis.call("LLEN", src_window)
local summary_len = redis.call("LLEN", src_summary)
if window_len == 0 and summary_len == 0 then
  redis.call("DEL", src_tool)
  redis.call("DEL", metadata_key)
  return {0, 0, 0}
end

redis.call("DEL", dst_window, dst_summary, dst_tool)
if window_len > 0 then
  redis.call("RENAME", src_window, dst_window)
  if redis.call("EXISTS", src_tool) == 1 then
    redis.call("RENAME", src_tool, dst_tool)
  else
    redis.call("SET", dst_tool, "0")
  end
elseif redis.call("EXISTS", src_tool) == 1 then
  redis.call("DEL", src_tool)
end
if summary_len > 0 then
  redis.call("RENAME", src_summary, dst_summary)
end
redis.call("DEL", metadata_key)

return {1, window_len, summary_len}
"#;

        let (restored, window_len, summary_len) = self
            .run_command::<(i64, usize, usize), _>("atomic_resume_bounded_snapshot", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script)
                    .arg(7)
                    .arg(&src_window)
                    .arg(&src_summary)
                    .arg(&src_tool)
                    .arg(&dst_window)
                    .arg(&dst_summary)
                    .arg(&dst_tool)
                    .arg(&metadata_key);
                cmd
            })
            .await?;

        if restored == 0 {
            tracing::debug!(
                event = SessionEvent::ContextWindowResumeMissing.as_str(),
                session_id,
                backup_session_id,
                backend = "valkey",
                "atomic bounded context resume skipped: no snapshot"
            );
            return Ok(None);
        }

        tracing::debug!(
            event = SessionEvent::ContextWindowResumed.as_str(),
            session_id,
            backup_session_id,
            messages = window_len,
            summary_segments = summary_len,
            backend = "valkey",
            "atomic bounded context snapshot resumed"
        );
        Ok(Some((window_len, summary_len)))
    }

    pub(crate) async fn atomic_drop_bounded_snapshot(
        &self,
        backup_session_id: &str,
        metadata_session_id: &str,
    ) -> Result<bool> {
        let backup_window = self.window_key(backup_session_id);
        let backup_summary = self.summary_key(backup_session_id);
        let backup_tool = self.stream_metrics_session_key("window_tool_calls", backup_session_id);
        let metadata_key = self.messages_key(metadata_session_id);
        let script = r#"
local backup_window = KEYS[1]
local backup_summary = KEYS[2]
local backup_tool = KEYS[3]
local metadata_key = KEYS[4]

local window_len = redis.call("LLEN", backup_window)
local summary_len = redis.call("LLEN", backup_summary)
redis.call("DEL", backup_window, backup_summary, backup_tool, metadata_key)
if window_len > 0 or summary_len > 0 then
  return 1
end
return 0
"#;

        let dropped = self
            .run_command::<i64, _>("atomic_drop_bounded_snapshot", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script)
                    .arg(4)
                    .arg(&backup_window)
                    .arg(&backup_summary)
                    .arg(&backup_tool)
                    .arg(&metadata_key);
                cmd
            })
            .await?;

        Ok(dropped == 1)
    }
}
