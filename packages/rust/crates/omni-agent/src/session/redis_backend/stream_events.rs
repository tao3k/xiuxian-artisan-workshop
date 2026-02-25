use anyhow::{Context, Result};

use crate::observability::SessionEvent;

use super::RedisSessionBackend;

const DEFAULT_STREAM_MAX_LEN: usize = 10_000;
const PUBLISH_STREAM_EVENT_SCRIPT: &str = r#"
local stream_key = KEYS[1]
local global_metrics_key = KEYS[2]
local session_metrics_key = KEYS[3]

local max_len = tonumber(ARGV[1]) or 10000
local ttl_secs = tonumber(ARGV[2]) or 0
local updated_at_unix_ms = tostring(ARGV[3])
local kind = ARGV[4]
local session_id = ARGV[5]
local field_count = tonumber(ARGV[6]) or 0

if field_count <= 0 then
  return redis.error_reply("stream event fields must not be empty")
end

local entries = {}
for i = 1, field_count do
  local offset = 6 + ((i - 1) * 2)
  entries[(i - 1) * 2 + 1] = ARGV[offset + 1]
  entries[(i - 1) * 2 + 2] = ARGV[offset + 2]
end

local event_id = redis.call("XADD", stream_key, "MAXLEN", "~", max_len, "*", unpack(entries))
redis.call("HINCRBY", global_metrics_key, "events_total", 1)
redis.call("HINCRBY", global_metrics_key, "kind:" .. kind, 1)
redis.call(
  "HSET",
  global_metrics_key,
  "last_event_id",
  event_id,
  "last_kind",
  kind,
  "updated_at_unix_ms",
  updated_at_unix_ms
)

if session_id ~= "" then
  redis.call("HINCRBY", session_metrics_key, "events_total", 1)
  redis.call("HINCRBY", session_metrics_key, "kind:" .. kind, 1)
  redis.call(
    "HSET",
    session_metrics_key,
    "last_event_id",
    event_id,
    "last_kind",
    kind,
    "updated_at_unix_ms",
    updated_at_unix_ms
  )
end

if ttl_secs > 0 then
  redis.call("EXPIRE", stream_key, ttl_secs)
  redis.call("EXPIRE", global_metrics_key, ttl_secs)
  if session_id ~= "" then
    redis.call("EXPIRE", session_metrics_key, ttl_secs)
  end
end

return event_id
"#;

#[derive(Debug, Clone)]
struct StreamEventPublishContext {
    stream_key: String,
    global_metrics_key: String,
    session_metrics_key: String,
    kind: String,
    session_id: String,
}

impl RedisSessionBackend {
    pub(crate) async fn publish_stream_event(
        &self,
        stream_name: &str,
        fields: &[(String, String)],
    ) -> Result<String> {
        validate_stream_event_publish_input(stream_name, fields)?;
        let publish_context = self.build_stream_event_publish_context(stream_name, fields);
        let ttl_secs = self.ttl_secs.unwrap_or(0);
        let now_unix_ms = Self::now_unix_ms();
        let field_count = i64::try_from(fields.len())
            .context("stream event fields count overflow for valkey stream publish")?;

        let event_id = self
            .run_command::<String, _>("publish_stream_event", || {
                build_publish_stream_event_cmd(
                    &publish_context,
                    fields,
                    ttl_secs,
                    now_unix_ms,
                    field_count,
                )
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionStreamEventPublished.as_str(),
            stream_name,
            stream_key = %publish_context.stream_key,
            global_metrics_key = %publish_context.global_metrics_key,
            session_metrics_key = %publish_context.session_metrics_key,
            kind = %publish_context.kind,
            session_id = %publish_context.session_id,
            event_id = %event_id,
            fields = fields.len(),
            "valkey stream event published"
        );
        Ok(event_id)
    }

    fn build_stream_event_publish_context(
        &self,
        stream_name: &str,
        fields: &[(String, String)],
    ) -> StreamEventPublishContext {
        let kind = find_stream_field(fields, "kind").unwrap_or_else(|| "unknown".to_string());
        let session_id = find_stream_field(fields, "session_id").unwrap_or_default();
        let session_metrics_key = if session_id.is_empty() {
            self.stream_metrics_session_key(stream_name, "__none__")
        } else {
            self.stream_metrics_session_key(stream_name, &session_id)
        };
        StreamEventPublishContext {
            stream_key: self.stream_key(stream_name),
            global_metrics_key: self.stream_metrics_global_key(stream_name),
            session_metrics_key,
            kind,
            session_id,
        }
    }
}

fn validate_stream_event_publish_input(
    stream_name: &str,
    fields: &[(String, String)],
) -> Result<()> {
    if stream_name.trim().is_empty() {
        anyhow::bail!("stream_name must not be empty");
    }
    if fields.is_empty() {
        anyhow::bail!("stream event fields must not be empty");
    }
    Ok(())
}

fn find_stream_field(fields: &[(String, String)], field_name: &str) -> Option<String> {
    fields
        .iter()
        .find_map(|(field, value)| (field == field_name).then_some(value.clone()))
}

fn build_publish_stream_event_cmd(
    publish_context: &StreamEventPublishContext,
    fields: &[(String, String)],
    ttl_secs: u64,
    now_unix_ms: u64,
    field_count: i64,
) -> redis::Cmd {
    let mut cmd = redis::cmd("EVAL");
    cmd.arg(PUBLISH_STREAM_EVENT_SCRIPT)
        .arg(3)
        .arg(&publish_context.stream_key)
        .arg(&publish_context.global_metrics_key)
        .arg(&publish_context.session_metrics_key)
        .arg(DEFAULT_STREAM_MAX_LEN)
        .arg(ttl_secs)
        .arg(now_unix_ms)
        .arg(&publish_context.kind)
        .arg(&publish_context.session_id)
        .arg(field_count);
    for (field, value) in fields {
        cmd.arg(field).arg(value);
    }
    cmd
}
