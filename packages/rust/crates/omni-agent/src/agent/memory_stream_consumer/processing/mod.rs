use crate::agent::logging::should_surface_repeated_failure;
use crate::observability::SessionEvent;

use super::types::{
    MemoryStreamConsumerRuntimeConfig, MemoryStreamEvent, StreamEventLogContext,
    bump_failure_streak_and_backoff, field_value_or_default,
};

mod ack_metrics;
mod promotion;

pub(super) use ack_metrics::ack_and_record_metrics;
pub(super) use promotion::queue_promoted_candidate;

pub(super) async fn process_stream_events(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    events: Vec<MemoryStreamEvent>,
    read_failure_streak: &mut u32,
) -> bool {
    for event in events {
        if !process_stream_event(connection, config, &event, read_failure_streak).await {
            return true;
        }
    }
    false
}

async fn process_stream_event(
    connection: &mut redis::aio::MultiplexedConnection,
    config: &MemoryStreamConsumerRuntimeConfig,
    event: &MemoryStreamEvent,
    read_failure_streak: &mut u32,
) -> bool {
    let kind = field_value_or_default(&event.fields, "kind", "unknown");
    let session_id = super::types::non_empty_string(event.fields.get("session_id").cloned());
    let context = StreamEventLogContext {
        event_id: &event.id,
        kind: &kind,
        session_id: session_id.as_deref(),
    };
    let mut promotion_queued: Option<bool> = None;

    if kind == "memory_promoted" {
        match queue_promoted_candidate(connection, config, event).await {
            Ok(inserted) => {
                promotion_queued = Some(inserted);
            }
            Err(error) => {
                let retry_backoff_ms = bump_failure_streak_and_backoff(read_failure_streak);
                log_stream_event_failure(
                    config,
                    &context,
                    *read_failure_streak,
                    retry_backoff_ms,
                    &error,
                    "memory stream consumer failed to queue promoted candidate",
                );
                return false;
            }
        }
    }

    match ack_and_record_metrics(connection, config, &event.id, &kind, session_id.as_deref()).await
    {
        Ok(acked) => {
            tracing::debug!(
                event = SessionEvent::MemoryStreamConsumerEventProcessed.as_str(),
                stream_name = %config.stream_name,
                stream_consumer_group = %config.stream_consumer_group,
                stream_consumer_name = %config.stream_consumer_name,
                event_id = %event.id,
                kind = %kind,
                session_id = session_id.as_deref().unwrap_or(""),
                acked,
                promotion_queued = ?promotion_queued,
                "memory stream event processed"
            );
            true
        }
        Err(error) => {
            let retry_backoff_ms = bump_failure_streak_and_backoff(read_failure_streak);
            log_stream_event_failure(
                config,
                &context,
                *read_failure_streak,
                retry_backoff_ms,
                &error,
                "memory stream consumer failed to ack/record event",
            );
            false
        }
    }
}

fn log_stream_event_failure(
    config: &MemoryStreamConsumerRuntimeConfig,
    context: &StreamEventLogContext<'_>,
    failure_streak: u32,
    retry_backoff_ms: u64,
    error: &anyhow::Error,
    message: &str,
) {
    if should_surface_repeated_failure(failure_streak) {
        tracing::warn!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            event_id = context.event_id,
            kind = context.kind,
            session_id = context.session_id.unwrap_or(""),
            failure_streak,
            retry_backoff_ms,
            error = %error,
            "{message}"
        );
    } else {
        tracing::trace!(
            event = SessionEvent::MemoryStreamConsumerReadFailed.as_str(),
            stream_name = %config.stream_name,
            stream_consumer_group = %config.stream_consumer_group,
            stream_consumer_name = %config.stream_consumer_name,
            event_id = context.event_id,
            kind = context.kind,
            session_id = context.session_id.unwrap_or(""),
            failure_streak,
            retry_backoff_ms,
            error = %error,
            "{message}"
        );
    }
}
