use anyhow::{Context, Result};
use serde::Deserialize;

use crate::observability::SessionEvent;

use super::super::message::ChatMessage;
use super::RedisSessionBackend;

const SESSION_CONTEXT_BACKUP_META_PREFIX: &str = "__session_context_backup_meta__:";

#[derive(Debug, Deserialize)]
struct LegacySessionContextBackupMetadataPayload {
    #[allow(dead_code)]
    messages: usize,
    #[allow(dead_code)]
    summary_segments: usize,
    #[allow(dead_code)]
    saved_at_unix_ms: u64,
}

impl RedisSessionBackend {
    pub(crate) async fn append_messages(
        &self,
        session_id: &str,
        messages: &[ChatMessage],
    ) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }
        let key = self.messages_key(session_id);
        let encoded: Vec<String> = messages
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to encode chat messages for redis")?;
        let ttl_secs = self.ttl_secs;

        self.run_pipeline::<(), _>("append_messages", || {
            let mut pipe = redis::pipe();
            pipe.atomic();
            pipe.cmd("RPUSH").arg(&key);
            for payload in &encoded {
                pipe.arg(payload);
            }
            pipe.ignore();
            if let Some(ttl) = ttl_secs {
                pipe.cmd("EXPIRE").arg(&key).arg(ttl).ignore();
            }
            pipe
        })
        .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesAppended.as_str(),
            session_id,
            appended_messages = encoded.len(),
            ttl_secs = ?ttl_secs,
            "valkey session messages appended"
        );
        Ok(())
    }

    pub(crate) async fn replace_messages(
        &self,
        session_id: &str,
        messages: &[ChatMessage],
    ) -> Result<usize> {
        let key = self.messages_key(session_id);
        let encoded: Vec<String> = messages
            .iter()
            .map(serde_json::to_string)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to encode chat messages for redis replace")?;
        let ttl_secs = self.ttl_secs.unwrap_or(0);
        let message_count = encoded.len();
        let message_count_i64 = i64::try_from(message_count)
            .context("message count overflow while replacing redis session messages")?;
        let script = r#"
local key = KEYS[1]
local ttl = tonumber(ARGV[1]) or 0
local count = tonumber(ARGV[2]) or 0
redis.call("DEL", key)
if count > 0 then
  for i = 1, count do
    redis.call("RPUSH", key, ARGV[2 + i])
  end
  if ttl > 0 then
    redis.call("EXPIRE", key, ttl)
  end
end
return count
"#;

        let replaced_count = self
            .run_command::<usize, _>("replace_messages", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script)
                    .arg(1)
                    .arg(&key)
                    .arg(ttl_secs)
                    .arg(message_count_i64);
                for payload in &encoded {
                    cmd.arg(payload);
                }
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesReplaced.as_str(),
            session_id,
            replaced_messages = replaced_count,
            ttl_secs = ?self.ttl_secs,
            "valkey session messages replaced atomically"
        );
        Ok(replaced_count)
    }

    pub(crate) async fn get_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let key = self.messages_key(session_id);
        let payloads = self
            .run_command::<Vec<String>, _>("get_messages", || {
                let mut cmd = redis::cmd("LRANGE");
                cmd.arg(&key).arg(0).arg(-1);
                cmd
            })
            .await?;
        let mut out = Vec::with_capacity(payloads.len());
        let mut invalid_payloads = 0usize;
        for payload in payloads {
            match decode_chat_message_payload(session_id, &payload) {
                Ok(message) => out.push(message),
                Err(error) => {
                    invalid_payloads += 1;
                    tracing::warn!(
                        event = SessionEvent::SessionMessagesLoaded.as_str(),
                        session_id,
                        error = %error,
                        "invalid chat message payload in redis session store"
                    );
                }
            }
        }
        tracing::debug!(
            event = SessionEvent::SessionMessagesLoaded.as_str(),
            session_id,
            loaded_messages = out.len(),
            invalid_payloads,
            "valkey session messages loaded"
        );
        Ok(out)
    }

    pub(crate) async fn get_messages_len(&self, session_id: &str) -> Result<usize> {
        let key = self.messages_key(session_id);
        let message_count = self
            .run_command::<usize, _>("get_messages_len", || {
                let mut cmd = redis::cmd("LLEN");
                cmd.arg(&key);
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesLoaded.as_str(),
            session_id,
            loaded_messages = message_count,
            count_only = true,
            "valkey session message count loaded"
        );
        Ok(message_count)
    }

    pub(crate) async fn clear_messages(&self, session_id: &str) -> Result<()> {
        let key = self.messages_key(session_id);
        let _ = self
            .run_command::<i64, _>("clear_messages", || {
                let mut cmd = redis::cmd("DEL");
                cmd.arg(&key);
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionMessagesCleared.as_str(),
            session_id,
            "valkey session messages cleared"
        );
        Ok(())
    }
}

fn decode_chat_message_payload(
    session_id: &str,
    payload: &str,
) -> std::result::Result<ChatMessage, serde_json::Error> {
    match serde_json::from_str::<ChatMessage>(payload) {
        Ok(message) => Ok(message),
        Err(chat_message_error) if session_id.starts_with(SESSION_CONTEXT_BACKUP_META_PREFIX) => {
            match serde_json::from_str::<LegacySessionContextBackupMetadataPayload>(payload) {
                Ok(_) => Ok(ChatMessage {
                    role: "system".to_string(),
                    content: Some(payload.to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                }),
                Err(_) => Err(chat_message_error),
            }
        }
        Err(chat_message_error) => Err(chat_message_error),
    }
}
