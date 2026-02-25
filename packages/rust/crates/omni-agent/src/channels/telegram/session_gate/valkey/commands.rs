use anyhow::{Context, Result};
use redis::FromRedisValue;

use crate::observability::SessionEvent;

use super::ValkeySessionGateBackend;

impl ValkeySessionGateBackend {
    pub(super) async fn try_acquire_lease(
        &self,
        lock_key: &str,
        owner_token: &str,
    ) -> Result<bool> {
        let acquired = self
            .run_command::<Option<String>, _>("session_gate_try_acquire", || {
                let mut cmd = redis::cmd("SET");
                cmd.arg(lock_key)
                    .arg(owner_token)
                    .arg("NX")
                    .arg("PX")
                    .arg(self.lease_ttl_ms);
                cmd
            })
            .await?;
        Ok(acquired.is_some())
    }

    pub(super) async fn renew_lease(&self, lock_key: &str, owner_token: &str) -> Result<bool> {
        let script = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("PEXPIRE", KEYS[1], ARGV[2])
else
  return 0
end
"#;
        let renewed = self
            .run_command::<i64, _>("session_gate_renew_lease", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script)
                    .arg(1)
                    .arg(lock_key)
                    .arg(owner_token)
                    .arg(self.lease_ttl_ms);
                cmd
            })
            .await?;
        Ok(renewed == 1)
    }

    pub(super) async fn release_lease(&self, lock_key: &str, owner_token: &str) -> Result<bool> {
        let script = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("DEL", KEYS[1])
else
  return 0
end
"#;
        let released = self
            .run_command::<i64, _>("session_gate_release_lease", || {
                let mut cmd = redis::cmd("EVAL");
                cmd.arg(script).arg(1).arg(lock_key).arg(owner_token);
                cmd
            })
            .await?;
        Ok(released == 1)
    }

    async fn run_command<T, F>(&self, operation: &'static str, build: F) -> Result<T>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..2 {
            let mut conn_guard = self.connection.lock().await;
            self.ensure_connection(&mut conn_guard).await?;
            let conn = conn_guard
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("session gate valkey connection unavailable"))?;
            let cmd = build();
            let result: redis::RedisResult<T> = cmd.query_async(conn).await;
            match result {
                Ok(value) => {
                    if attempt > 0 {
                        tracing::debug!(
                            event = SessionEvent::SessionValkeyCommandRetrySucceeded.as_str(),
                            operation,
                            attempt = attempt + 1,
                            "session gate valkey command succeeded after retry"
                        );
                    }
                    return Ok(value);
                }
                Err(err) => {
                    tracing::warn!(
                        event = SessionEvent::SessionValkeyCommandRetryFailed.as_str(),
                        operation,
                        attempt = attempt + 1,
                        error = %err,
                        "session gate valkey command failed; reconnecting"
                    );
                    *conn_guard = None;
                    last_err =
                        Some(anyhow::anyhow!(err).context("session gate valkey command failed"));
                }
            }
        }
        tracing::warn!(
            event = SessionEvent::SessionValkeyCommandRetryFailed.as_str(),
            operation,
            "session gate valkey command failed after retry"
        );
        Err(last_err
            .unwrap_or_else(|| anyhow::anyhow!("session gate valkey command failed unexpectedly")))
    }

    async fn ensure_connection(
        &self,
        connection: &mut Option<redis::aio::MultiplexedConnection>,
    ) -> Result<()> {
        if connection.is_some() {
            return Ok(());
        }
        *connection = Some(
            self.client
                .get_multiplexed_async_connection()
                .await
                .context("failed to open valkey connection for session gate")?,
        );
        tracing::debug!(
            event = SessionEvent::SessionValkeyConnected.as_str(),
            key_prefix = %self.key_prefix,
            "valkey session gate backend connected"
        );
        Ok(())
    }
}
