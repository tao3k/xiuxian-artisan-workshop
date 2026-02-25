mod bootstrap;
mod parsing;
mod processing;
mod runtime;
mod stream;
mod types;

pub(super) use bootstrap::spawn_memory_stream_consumer;

#[cfg(test)]
use parsing::parse_xreadgroup_reply;
#[cfg(test)]
use processing::{ack_and_record_metrics, queue_promoted_candidate};
#[cfg(test)]
use runtime::classify_stream_read_error;
#[cfg(test)]
use stream::ensure_consumer_group;
#[cfg(test)]
use stream::{
    is_idle_poll_timeout_error, read_stream_events, stream_consumer_connection_config,
    stream_consumer_response_timeout, summarize_redis_error,
};
#[cfg(test)]
use types::compute_retry_backoff_ms;
#[cfg(test)]
use types::{MemoryStreamConsumerRuntimeConfig, StreamReadErrorKind, build_consumer_name};

#[cfg(test)]
#[path = "../../../tests/agent/memory_stream_consumer.rs"]
mod tests;
