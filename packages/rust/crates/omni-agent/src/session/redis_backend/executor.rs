use anyhow::{Context, Result};
use redis::FromRedisValue;

use crate::observability::SessionEvent;

use super::RedisSessionBackend;

impl RedisSessionBackend {
    async fn acquire_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let _reconnect_guard = self.reconnect_lock.lock().await;
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("failed to open redis connection for session backend")?;
        {
            let mut guard = self.connection.write().await;
            *guard = Some(connection.clone());
        }
        tracing::debug!(
            event = SessionEvent::SessionValkeyConnected.as_str(),
            key_prefix = %self.key_prefix,
            "valkey session backend connected"
        );
        Ok(connection)
    }

    async fn invalidate_connection(&self) {
        let mut guard = self.connection.write().await;
        *guard = None;
    }

    pub(super) async fn run_command<T, F>(&self, operation: &'static str, build: F) -> Result<T>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_err: Option<anyhow::Error> = None;
        for attempt in 0..2 {
            let mut conn = self.acquire_connection().await?;
            let cmd = build();
            let result: redis::RedisResult<T> = cmd.query_async(&mut conn).await;
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
                    self.invalidate_connection().await;
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
            let mut conn = self.acquire_connection().await?;
            let pipe = build();
            let result: redis::RedisResult<T> = pipe.query_async(&mut conn).await;
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
                    self.invalidate_connection().await;
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
