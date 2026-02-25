use anyhow::{Context, Result};
use redis::FromRedisValue;

use crate::observability::SessionEvent;

use super::RedisSessionBackend;

impl RedisSessionBackend {
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
                .context("failed to open redis connection for session backend")?,
        );
        tracing::debug!(
            event = SessionEvent::SessionValkeyConnected.as_str(),
            key_prefix = %self.key_prefix,
            "valkey session backend connected"
        );
        Ok(())
    }

    pub(super) async fn run_command<T, F>(&self, operation: &'static str, build: F) -> Result<T>
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
                .ok_or_else(|| anyhow::anyhow!("redis session backend connection unavailable"))?;
            let cmd = build();
            let result: redis::RedisResult<T> = cmd.query_async(conn).await;
            match result {
                Ok(value) => {
                    if attempt > 0 {
                        tracing::debug!(
                            event = SessionEvent::SessionValkeyCommandRetrySucceeded.as_str(),
                            operation,
                            attempt = attempt + 1,
                            "valkey command succeeded after retry"
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
                        "valkey command attempt failed; reconnecting"
                    );
                    *conn_guard = None;
                    last_err = Some(
                        anyhow::anyhow!(err).context("redis command failed for session backend"),
                    );
                }
            }
        }
        tracing::warn!(
            event = SessionEvent::SessionValkeyCommandRetryFailed.as_str(),
            operation,
            "valkey command failed after retry"
        );
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("redis command failed for unknown reason")))
    }

    pub(super) async fn run_pipeline<T, F>(&self, operation: &'static str, build: F) -> Result<T>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Pipeline,
    {
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..2 {
            let mut conn_guard = self.connection.lock().await;
            self.ensure_connection(&mut conn_guard).await?;
            let conn = conn_guard
                .as_mut()
                .ok_or_else(|| anyhow::anyhow!("redis session backend connection unavailable"))?;
            let pipe = build();
            let result: redis::RedisResult<T> = pipe.query_async(conn).await;
            match result {
                Ok(value) => {
                    if attempt > 0 {
                        tracing::debug!(
                            event = SessionEvent::SessionValkeyPipelineRetrySucceeded.as_str(),
                            operation,
                            attempt = attempt + 1,
                            "valkey pipeline succeeded after retry"
                        );
                    }
                    return Ok(value);
                }
                Err(err) => {
                    tracing::warn!(
                        event = SessionEvent::SessionValkeyPipelineRetryFailed.as_str(),
                        operation,
                        attempt = attempt + 1,
                        error = %err,
                        "valkey pipeline attempt failed; reconnecting"
                    );
                    *conn_guard = None;
                    last_err = Some(
                        anyhow::anyhow!(err).context("redis pipeline failed for session backend"),
                    );
                }
            }
        }
        tracing::warn!(
            event = SessionEvent::SessionValkeyPipelineRetryFailed.as_str(),
            operation,
            "valkey pipeline failed after retry"
        );
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("redis pipeline failed for unknown reason")))
    }
}
