use tokio::task::JoinHandle;

use crate::config::MemoryConfig;
use crate::observability::SessionEvent;
use crate::session::RedisSessionRuntimeSnapshot;

use super::runtime::run_consumer_loop;
use super::types::{
    DEFAULT_CONSUMER_GROUP, DEFAULT_CONSUMER_PREFIX, DEFAULT_STREAM_NAME,
    MemoryStreamConsumerRuntimeConfig, build_consumer_name, non_empty_string,
};

pub(in crate::agent) fn spawn_memory_stream_consumer(
    memory_cfg: &MemoryConfig,
    session_redis: Option<RedisSessionRuntimeSnapshot>,
) -> Option<JoinHandle<()>> {
    if !memory_cfg.stream_consumer_enabled {
        tracing::info!(
            event = SessionEvent::MemoryStreamConsumerDisabled.as_str(),
            reason = "disabled_by_config",
            "memory stream consumer disabled"
        );
        return None;
    }

    let Some(session_redis) = session_redis else {
        tracing::info!(
            event = SessionEvent::MemoryStreamConsumerDisabled.as_str(),
            reason = "session_valkey_backend_unavailable",
            "memory stream consumer disabled"
        );
        return None;
    };

    let runtime_cfg = build_runtime_config(memory_cfg, &session_redis);
    log_stream_consumer_start(&runtime_cfg);

    Some(tokio::spawn(async move {
        run_consumer_loop(runtime_cfg).await;
    }))
}

fn build_runtime_config(
    memory_cfg: &MemoryConfig,
    session_redis: &RedisSessionRuntimeSnapshot,
) -> MemoryStreamConsumerRuntimeConfig {
    let stream_name = non_empty_string(Some(memory_cfg.stream_name.clone()))
        .unwrap_or_else(|| DEFAULT_STREAM_NAME.to_string());
    let stream_consumer_group = non_empty_string(Some(memory_cfg.stream_consumer_group.clone()))
        .unwrap_or_else(|| DEFAULT_CONSUMER_GROUP.to_string());
    let stream_consumer_name_prefix =
        non_empty_string(Some(memory_cfg.stream_consumer_name_prefix.clone()))
            .unwrap_or_else(|| DEFAULT_CONSUMER_PREFIX.to_string());
    let stream_consumer_name = build_consumer_name(&stream_consumer_name_prefix);
    let stream_consumer_batch_size = memory_cfg.stream_consumer_batch_size.max(1);
    let stream_consumer_block_ms = memory_cfg.stream_consumer_block_ms.max(1);

    MemoryStreamConsumerRuntimeConfig {
        redis_url: session_redis.url.clone(),
        stream_key: format!("{}:stream:{stream_name}", session_redis.key_prefix),
        promotion_stream_key: format!(
            "{}:stream:knowledge.ingest.candidates",
            session_redis.key_prefix
        ),
        promotion_ledger_key: format!("{}:knowledge:ingest:candidates", session_redis.key_prefix),
        stream_name: stream_name.clone(),
        stream_consumer_group,
        stream_consumer_name,
        stream_consumer_batch_size,
        stream_consumer_block_ms,
        metrics_global_key: format!(
            "{}:metrics:{stream_name}:consumer",
            session_redis.key_prefix
        ),
        metrics_session_prefix: format!(
            "{}:metrics:{stream_name}:consumer:session:",
            session_redis.key_prefix
        ),
        ttl_secs: session_redis.ttl_secs,
    }
}

fn log_stream_consumer_start(config: &MemoryStreamConsumerRuntimeConfig) {
    tracing::info!(
        event = SessionEvent::MemoryStreamConsumerStarted.as_str(),
        stream_name = %config.stream_name,
        stream_key = %config.stream_key,
        promotion_stream_key = %config.promotion_stream_key,
        promotion_ledger_key = %config.promotion_ledger_key,
        stream_consumer_group = %config.stream_consumer_group,
        stream_consumer_name = %config.stream_consumer_name,
        stream_consumer_batch_size = config.stream_consumer_batch_size,
        stream_consumer_block_ms = config.stream_consumer_block_ms,
        "memory stream consumer task starting"
    );
}
