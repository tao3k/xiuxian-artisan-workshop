use std::time::Instant;

use super::chunk_dispatch::{
    collect_concurrent_chunk_result, dispatch_chunk_with_runtime, merge_concurrent_chunk_results,
    spawn_chunk_task,
};
use super::support::build_chunk_ranges;
use super::{EmbeddingClient, EmbeddingDispatchRuntime};

impl EmbeddingClient {
    /// Embed texts with an optional embedding model hint.
    pub async fn embed_batch_with_model(
        &self,
        texts: &[String],
        model: Option<&str>,
    ) -> Option<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Some(vec![]);
        }
        let resolved_model = model
            .map(str::trim)
            .map(ToString::to_string)
            .filter(|value| !value.is_empty())
            .or_else(|| self.default_model.clone());
        let started = Instant::now();
        if let Some(cached) = self.cache.get_batch(texts, resolved_model.as_deref()).await {
            tracing::debug!(
                event = "agent.embedding.cache.hit",
                batch_size = texts.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "embedding batch served from local cache"
            );
            return Some(cached);
        }
        let chunk_ranges = build_chunk_ranges(texts.len(), self.batch_max_size);
        let chunk_count = chunk_ranges.len();
        let effective_chunk_concurrency = self.batch_max_concurrency.max(1).min(chunk_count.max(1));
        tracing::debug!(
            event = "agent.embedding.batch.plan",
            backend = self.backend_mode.as_str(),
            backend_source = self.backend_source,
            batch_size = texts.len(),
            model = resolved_model.as_deref().unwrap_or(""),
            chunk_count,
            chunk_max_size = self.batch_max_size,
            chunk_concurrency = effective_chunk_concurrency,
            max_in_flight = self.max_in_flight,
            "embedding batch execution plan prepared"
        );

        let runtime = self.dispatch_runtime();
        let result = self
            .dispatch_embeddings_for_ranges(
                &runtime,
                texts,
                resolved_model.as_deref(),
                &chunk_ranges,
                effective_chunk_concurrency,
            )
            .await;

        if let Some(vectors) = result.as_ref() {
            self.cache
                .put_batch(texts, vectors, resolved_model.as_deref())
                .await;
        }
        tracing::debug!(
            event = "agent.embedding.batch.completed",
            backend = self.backend_mode.as_str(),
            backend_source = self.backend_source,
            success = result.is_some(),
            elapsed_ms = started.elapsed().as_millis(),
            "embedding batch completed"
        );
        result
    }

    fn dispatch_runtime(&self) -> EmbeddingDispatchRuntime {
        EmbeddingDispatchRuntime {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            mcp_url: self.mcp_url.clone(),
            backend_mode: self.backend_mode,
            backend_source: self.backend_source,
            #[cfg(feature = "agent-provider-litellm")]
            timeout_secs: self.timeout_secs,
            max_in_flight: self.max_in_flight,
            in_flight_gate: self.in_flight_gate.clone(),
            mistral_sdk_hf_cache_path: self.mistral_sdk_hf_cache_path.clone(),
            mistral_sdk_hf_revision: self.mistral_sdk_hf_revision.clone(),
            mistral_sdk_max_num_seqs: self.mistral_sdk_max_num_seqs,
            #[cfg(feature = "agent-provider-litellm")]
            litellm_api_key: self.litellm_api_key.clone(),
        }
    }

    async fn dispatch_embeddings_for_ranges(
        &self,
        runtime: &EmbeddingDispatchRuntime,
        texts: &[String],
        model: Option<&str>,
        chunk_ranges: &[(usize, usize)],
        chunk_concurrency: usize,
    ) -> Option<Vec<Vec<f32>>> {
        if chunk_ranges.is_empty() {
            return Some(vec![]);
        }
        if chunk_ranges.len() == 1 {
            let vectors = dispatch_chunk_with_runtime(runtime, texts, model, 0, 1).await?;
            if vectors.len() != texts.len() {
                tracing::warn!(
                    event = "agent.embedding.batch.invalid_vector_count",
                    expected_vectors = texts.len(),
                    actual_vectors = vectors.len(),
                    chunk_count = 1,
                    "embedding backend returned unexpected vector count"
                );
                return None;
            }
            return Some(vectors);
        }

        if chunk_concurrency <= 1 {
            return self
                .dispatch_embeddings_sequential(runtime, texts, model, chunk_ranges)
                .await;
        }

        self.dispatch_embeddings_concurrent(runtime, texts, model, chunk_ranges, chunk_concurrency)
            .await
    }

    async fn dispatch_embeddings_sequential(
        &self,
        runtime: &EmbeddingDispatchRuntime,
        texts: &[String],
        model: Option<&str>,
        chunk_ranges: &[(usize, usize)],
    ) -> Option<Vec<Vec<f32>>> {
        let chunk_count = chunk_ranges.len();
        let mut merged = Vec::with_capacity(texts.len());
        for (chunk_index, (start, end)) in chunk_ranges.iter().copied().enumerate() {
            let chunk = &texts[start..end];
            let vectors =
                dispatch_chunk_with_runtime(runtime, chunk, model, chunk_index, chunk_count)
                    .await?;
            if vectors.len() != chunk.len() {
                tracing::warn!(
                    event = "agent.embedding.batch.invalid_chunk_vector_count",
                    chunk_index = chunk_index + 1,
                    chunk_count,
                    expected_vectors = chunk.len(),
                    actual_vectors = vectors.len(),
                    "embedding backend returned unexpected chunk vector count"
                );
                return None;
            }
            merged.extend(vectors);
        }
        Some(merged)
    }

    async fn dispatch_embeddings_concurrent(
        &self,
        runtime: &EmbeddingDispatchRuntime,
        texts: &[String],
        model: Option<&str>,
        chunk_ranges: &[(usize, usize)],
        chunk_concurrency: usize,
    ) -> Option<Vec<Vec<f32>>> {
        let chunk_count = chunk_ranges.len();
        let concurrency = chunk_concurrency.max(1).min(chunk_count);
        tracing::debug!(
            event = "agent.embedding.batch.concurrent.start",
            chunk_count,
            chunk_concurrency = concurrency,
            "embedding chunked concurrent execution started"
        );

        let mut next_chunk = 0usize;
        let mut finished = 0usize;
        let mut pending = tokio::task::JoinSet::new();
        let mut chunk_results: Vec<Option<Vec<Vec<f32>>>> = vec![None; chunk_count];
        let model_owned = model.map(ToString::to_string);

        while next_chunk < concurrency {
            spawn_chunk_task(
                &mut pending,
                runtime,
                texts,
                chunk_ranges,
                model_owned.as_ref(),
                next_chunk,
                chunk_count,
            );
            next_chunk += 1;
        }

        while finished < chunk_count {
            let (chunk_index, vectors) =
                collect_concurrent_chunk_result(&mut pending, chunk_ranges, chunk_count, finished)
                    .await?;
            finished = finished.saturating_add(1);
            chunk_results[chunk_index] = Some(vectors);
            if next_chunk < chunk_count {
                spawn_chunk_task(
                    &mut pending,
                    runtime,
                    texts,
                    chunk_ranges,
                    model_owned.as_ref(),
                    next_chunk,
                    chunk_count,
                );
                next_chunk += 1;
            }
        }

        let merged = merge_concurrent_chunk_results(chunk_results, texts.len(), chunk_count)?;
        tracing::debug!(
            event = "agent.embedding.batch.concurrent.completed",
            chunk_count,
            chunk_concurrency = concurrency,
            merged_vectors = merged.len(),
            "embedding chunked concurrent execution completed"
        );
        Some(merged)
    }

    /// Embed single text with an optional embedding model hint.
    pub async fn embed_with_model(&self, text: &str, model: Option<&str>) -> Option<Vec<f32>> {
        let texts = [text.to_string()];
        self.embed_batch_with_model(&texts, model)
            .await
            .and_then(|batch| batch.into_iter().next())
    }
}
