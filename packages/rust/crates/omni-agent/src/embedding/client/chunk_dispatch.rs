use std::time::Instant;

use super::EmbeddingDispatchRuntime;
use super::backend_dispatch::dispatch_chunk_by_backend;

pub(super) async fn dispatch_chunk_with_runtime_owned(
    runtime: EmbeddingDispatchRuntime,
    texts: Vec<String>,
    model: Option<String>,
    chunk_index: usize,
    chunk_count: usize,
) -> (usize, Option<Vec<Vec<f32>>>) {
    let result =
        dispatch_chunk_with_runtime(&runtime, &texts, model.as_deref(), chunk_index, chunk_count)
            .await;
    (chunk_index, result)
}

pub(super) async fn dispatch_chunk_with_runtime(
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    model: Option<&str>,
    chunk_index: usize,
    chunk_count: usize,
) -> Option<Vec<Vec<f32>>> {
    let permit_context = acquire_in_flight_permit(runtime).await?;
    let _in_flight_permit = permit_context.permit;
    tracing::debug!(
        event = "agent.embedding.batch.dispatch",
        backend = runtime.backend_mode.as_str(),
        backend_source = runtime.backend_source,
        chunk_index = chunk_index + 1,
        chunk_count,
        chunk_size = texts.len(),
        model = model.unwrap_or(""),
        max_in_flight = runtime.max_in_flight,
        gate_wait_ms = permit_context.gate_wait_ms,
        gate_available_before = permit_context.gate_available_before,
        gate_available_after = permit_context.gate_available_after,
        "dispatching embedding batch chunk request"
    );

    dispatch_chunk_by_backend(runtime, texts, model).await
}

pub(super) type ChunkDispatchResult = (usize, Option<Vec<Vec<f32>>>);

struct InFlightPermitContext {
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
    gate_wait_ms: u64,
    gate_available_before: usize,
    gate_available_after: usize,
}

pub(super) fn spawn_chunk_task(
    pending: &mut tokio::task::JoinSet<ChunkDispatchResult>,
    runtime: &EmbeddingDispatchRuntime,
    texts: &[String],
    chunk_ranges: &[(usize, usize)],
    model: Option<&String>,
    chunk_index: usize,
    chunk_count: usize,
) {
    let (start, end) = chunk_ranges[chunk_index];
    let chunk_texts = texts[start..end].to_vec();
    pending.spawn(dispatch_chunk_with_runtime_owned(
        runtime.clone(),
        chunk_texts,
        model.cloned(),
        chunk_index,
        chunk_count,
    ));
}

pub(super) async fn collect_concurrent_chunk_result(
    pending: &mut tokio::task::JoinSet<ChunkDispatchResult>,
    chunk_ranges: &[(usize, usize)],
    chunk_count: usize,
    finished: usize,
) -> Option<(usize, Vec<Vec<f32>>)> {
    match pending.join_next().await {
        Some(Ok((chunk_index, Some(vectors)))) => {
            let (start, end) = chunk_ranges[chunk_index];
            let expected_vectors = end - start;
            if vectors.len() != expected_vectors {
                tracing::warn!(
                    event = "agent.embedding.batch.invalid_chunk_vector_count",
                    chunk_index = chunk_index + 1,
                    chunk_count,
                    expected_vectors,
                    actual_vectors = vectors.len(),
                    "embedding backend returned unexpected chunk vector count"
                );
                abort_pending_chunks(pending).await;
                return None;
            }
            Some((chunk_index, vectors))
        }
        Some(Ok((chunk_index, None))) => {
            tracing::warn!(
                event = "agent.embedding.batch.chunk_failed",
                chunk_index = chunk_index + 1,
                chunk_count,
                "embedding chunk failed during concurrent execution"
            );
            abort_pending_chunks(pending).await;
            None
        }
        Some(Err(error)) => {
            tracing::warn!(
                event = "agent.embedding.batch.chunk_join_failed",
                chunk_count,
                error = %error,
                "embedding chunk task join failed"
            );
            abort_pending_chunks(pending).await;
            None
        }
        None => {
            tracing::warn!(
                event = "agent.embedding.batch.chunk_join_unexpected_none",
                chunk_count,
                finished,
                "embedding chunk join set ended unexpectedly"
            );
            None
        }
    }
}

async fn abort_pending_chunks(pending: &mut tokio::task::JoinSet<ChunkDispatchResult>) {
    pending.abort_all();
    while pending.join_next().await.is_some() {}
}

pub(super) fn merge_concurrent_chunk_results(
    chunk_results: Vec<Option<Vec<Vec<f32>>>>,
    total_texts: usize,
    chunk_count: usize,
) -> Option<Vec<Vec<f32>>> {
    let mut merged = Vec::with_capacity(total_texts);
    for (chunk_index, chunk_vectors) in chunk_results.into_iter().enumerate() {
        let Some(vectors) = chunk_vectors else {
            tracing::warn!(
                event = "agent.embedding.batch.chunk_missing_result",
                chunk_index = chunk_index + 1,
                chunk_count,
                "embedding chunk result missing after concurrent execution"
            );
            return None;
        };
        merged.extend(vectors);
    }
    Some(merged)
}

async fn acquire_in_flight_permit(
    runtime: &EmbeddingDispatchRuntime,
) -> Option<InFlightPermitContext> {
    let gate_wait_started = Instant::now();
    let gate_available_before = runtime
        .in_flight_gate
        .as_ref()
        .map_or(0usize, |gate| gate.available_permits());
    let permit = if let Some(gate) = runtime.in_flight_gate.as_ref() {
        match gate.clone().acquire_owned().await {
            Ok(permit) => Some(permit),
            Err(error) => {
                tracing::warn!(
                    event = "agent.embedding.in_flight_gate.closed",
                    error = %error,
                    "embedding in-flight gate closed unexpectedly"
                );
                return None;
            }
        }
    } else {
        None
    };
    let gate_wait_ms = u64::try_from(gate_wait_started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let gate_available_after = runtime
        .in_flight_gate
        .as_ref()
        .map_or(0usize, |gate| gate.available_permits());
    Some(InFlightPermitContext {
        permit,
        gate_wait_ms,
        gate_available_before,
        gate_available_after,
    })
}
