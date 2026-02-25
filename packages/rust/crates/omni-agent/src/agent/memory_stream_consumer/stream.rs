use anyhow::Result;
use redis::Value;
use std::time::Duration;
use tokio::time::sleep;

use crate::agent::logging::should_surface_repeated_failure;
use crate::observability::SessionEvent;

use super::parsing::parse_xreadgroup_reply;
use super::types::{
    MemoryStreamConsumerRuntimeConfig, RECONNECT_BACKOFF_MS,
    STREAM_CONSUMER_RESPONSE_TIMEOUT_GRACE_MS, compute_retry_backoff_ms,
};

pub(super) fn open_stream_consumer_client(
    config: &MemoryStreamConsumerRuntimeConfig,
) -> Option<redis::Client> {
    match redis::Client::open(config.redis_url.as_str()) {
        Ok(client) => Some(client),
        Err(error) => {
            tracing::warn!(
                event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
                stream_name = %config.stream_name,
                error = %error,
                "memory stream consumer disabled due to invalid redis url"
            );
            None
        }
    }
}

pub(super) fn stream_consumer_response_timeout_ms(block_ms: u64) -> u64 {
    u64::try_from(
        stream_consumer_response_timeout(block_ms)
            .as_millis()
            .min(u128::from(u64::MAX)),
    )
    .unwrap_or(u64::MAX)
}

pub(super) async fn connect_stream_consumer(
    client: &redis::Client,
    connection_config: &redis::AsyncConnectionConfig,
    config: &MemoryStreamConsumerRuntimeConfig,
    response_timeout_ms: u64,
    connect_failure_streak: &mut u32,
) -> Option<redis::aio::MultiplexedConnection> {
    match client
        .get_multiplexed_async_connection_with_config(connection_config)
        .await
    {
        Ok(connection) => {
            *connect_failure_streak = 0;
            Some(connection)
        }
        Err(error) => {
            *connect_failure_streak = connect_failure_streak.saturating_add(1);
            let retry_backoff_ms =
                compute_retry_backoff_ms(RECONNECT_BACKOFF_MS, *connect_failure_streak);
            if should_surface_repeated_failure(*connect_failure_streak) {
                tracing::warn!(
                    event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
                    stream_name = %config.stream_name,
                    failure_streak = *connect_failure_streak,
                    retry_backoff_ms,
                    response_timeout_ms,
                    error = %error,
                    "memory stream consumer failed to connect to valkey; retrying"
                );
            } else {
                tracing::trace!(
                    event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
                    stream_name = %config.stream_name,
                    failure_streak = *connect_failure_streak,
                    retry_backoff_ms,
                    response_timeout_ms,
                    error = %error,
                    "memory stream consumer failed to connect to valkey; retrying"
                );
            }
            sleep(Duration::from_millis(retry_backoff_ms)).await;
            None
        }
    }
}

pub(super) async fn ensure_consumer_group_before_read(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    ensure_group_failure_streak: &mut u32,
) -> bool {
    match ensure_consumer_group(connection, config).await {
        Ok(()) => {
            *ensure_group_failure_streak = 0;
            true
        }
        Err(error) => {
            *ensure_group_failure_streak = ensure_group_failure_streak.saturating_add(1);
            let retry_backoff_ms =
                compute_retry_backoff_ms(RECONNECT_BACKOFF_MS, *ensure_group_failure_streak);
            if should_surface_repeated_failure(*ensure_group_failure_streak) {
                tracing::warn!(
                    event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
                    stream_name = %config.stream_name,
                    stream_consumer_group = %config.stream_consumer_group,
                    failure_streak = *ensure_group_failure_streak,
                    retry_backoff_ms,
                    error = %error,
                    "memory stream consumer failed to ensure consumer group; retrying"
                );
            } else {
                tracing::trace!(
                    event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
                    stream_name = %config.stream_name,
                    stream_consumer_group = %config.stream_consumer_group,
                    failure_streak = *ensure_group_failure_streak,
                    retry_backoff_ms,
                    error = %error,
                    "memory stream consumer failed to ensure consumer group; retrying"
                );
            }
            sleep(Duration::from_millis(retry_backoff_ms)).await;
            false
        }
    }
}

pub(super) async fn ensure_consumer_group(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
) -> Result<()> {
    let create_result: redis::RedisResult<String> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(&config.stream_key)
        .arg(&config.stream_consumer_group)
        .arg("0")
        .arg("MKSTREAM")
        .query_async(connection)
        .await;

    match create_result {
        Ok(_) => {
            tracing::info!(
                event = SessionEvent::MemoryStreamConsumerGroupReady.as_str(),
                stream_name = %config.stream_name,
                stream_key = %config.stream_key,
                stream_consumer_group = %config.stream_consumer_group,
                created = true,
                "memory stream consumer group created"
            );
            Ok(())
        }
        Err(error) if is_busy_group_error(&error) => {
            tracing::trace!(
                event = SessionEvent::MemoryStreamConsumerGroupReady.as_str(),
                stream_name = %config.stream_name,
                stream_key = %config.stream_key,
                stream_consumer_group = %config.stream_consumer_group,
                created = false,
                "memory stream consumer group already exists"
            );
            Ok(())
        }
        Err(error) => Err(anyhow::anyhow!("xgroup create failed: {error}")),
    }
}

pub(super) async fn read_stream_events(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    stream_id: &str,
) -> Result<Vec<super::types::MemoryStreamEvent>> {
    let mut command = redis::cmd("XREADGROUP");
    command
        .arg("GROUP")
        .arg(&config.stream_consumer_group)
        .arg(&config.stream_consumer_name)
        .arg("COUNT")
        .arg(config.stream_consumer_batch_size);
    if stream_id == ">" {
        command.arg("BLOCK").arg(config.stream_consumer_block_ms);
    }
    command
        .arg("STREAMS")
        .arg(&config.stream_key)
        .arg(stream_id);

    let response: Value = match command.query_async(connection).await {
        Ok(response) => response,
        Err(error) if stream_id == ">" && is_idle_poll_timeout_error(&error) => {
            tracing::trace!(
                event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
                stream_name = %config.stream_name,
                stream_consumer_group = %config.stream_consumer_group,
                stream_consumer_name = %config.stream_consumer_name,
                stream_id,
                error_kind = "read_timeout_treated_as_idle_poll",
                error = %error,
                "xreadgroup blocking poll timed out; treating as idle poll"
            );
            return Ok(Vec::new());
        }
        Err(error) => {
            let redis_error_summary = summarize_redis_error(&error);
            return Err(anyhow::anyhow!(
                "xreadgroup failed for stream_id={stream_id}: {redis_error_summary}"
            ));
        }
    };

    parse_xreadgroup_reply(response)
}

pub(super) fn stream_consumer_response_timeout(block_ms: u64) -> Duration {
    Duration::from_millis(
        block_ms
            .max(1)
            .saturating_add(STREAM_CONSUMER_RESPONSE_TIMEOUT_GRACE_MS),
    )
}

pub(super) fn stream_consumer_connection_config(block_ms: u64) -> redis::AsyncConnectionConfig {
    redis::AsyncConnectionConfig::new()
        .set_response_timeout(Some(stream_consumer_response_timeout(block_ms)))
}

fn is_busy_group_error(error: &redis::RedisError) -> bool {
    error.to_string().to_ascii_uppercase().contains("BUSYGROUP")
}

pub(super) fn summarize_redis_error(error: &redis::RedisError) -> String {
    let mut parts = Vec::with_capacity(6);
    parts.push(format!("kind={:?}", error.kind()));
    parts.push(format!("category={}", error.category()));
    parts.push(format!("timeout={}", error.is_timeout()));
    if let Some(code) = error.code().filter(|value| !value.trim().is_empty()) {
        parts.push(format!("code={code}"));
    }
    if let Some(detail) = error.detail().filter(|value| !value.trim().is_empty()) {
        parts.push(format!("detail={detail}"));
    }
    let display = error.to_string();
    if !display.trim().is_empty() {
        parts.push(format!("display={display}"));
    }
    format!("redis_error{{{}}}", parts.join(", "))
}

pub(super) fn is_idle_poll_timeout_error(error: &redis::RedisError) -> bool {
    if error.is_timeout() {
        return true;
    }
    let message = error.to_string().to_ascii_uppercase();
    [
        "TIMED OUT",
        "TIMEOUT",
        "DEADLINE HAS ELAPSED",
        "WOULDBLOCK",
        "WOULD BLOCK",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}
