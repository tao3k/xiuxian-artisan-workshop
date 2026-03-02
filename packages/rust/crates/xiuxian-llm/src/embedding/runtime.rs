//! Memory embedding runtime guard and vector repair utilities.

use num_traits::ToPrimitive;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Canonical source label for direct embedding output.
pub const EMBEDDING_SOURCE_EMBEDDING: &str = "embedding";
/// Canonical source label for repaired embedding output.
pub const EMBEDDING_SOURCE_EMBEDDING_REPAIRED: &str = "embedding_repaired";
/// Canonical source label for unavailable embedding output.
pub const EMBEDDING_SOURCE_UNAVAILABLE: &str = "embedding_unavailable";

/// Default timeout for one embedding request.
pub const DEFAULT_MEMORY_EMBED_TIMEOUT: Duration = Duration::from_secs(3);
/// Default cooldown after an embedding timeout.
pub const DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN: Duration = Duration::from_secs(20);
/// Lower bound for embedding timeout configuration.
pub const MIN_MEMORY_EMBED_TIMEOUT_MS: u64 = 100;
/// Upper bound for embedding timeout configuration.
pub const MAX_MEMORY_EMBED_TIMEOUT_MS: u64 = 60_000;
/// Upper bound for embedding timeout cooldown configuration.
pub const MAX_MEMORY_EMBED_COOLDOWN_MS: u64 = 300_000;

/// Embedding runtime failure category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryEmbeddingErrorKind {
    /// Request rejected because timeout cooldown is still active.
    CooldownActive,
    /// Request timed out.
    Timeout,
    /// Embedding backend returned no vector.
    Unavailable,
}

impl MemoryEmbeddingErrorKind {
    /// Return the canonical error label used by observability and snapshots.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CooldownActive => "cooldown_active",
            Self::Timeout => "timeout",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Stateful runtime guard for memory embedding operations.
///
/// The runtime applies timeout + cooldown policy and dimension repair in one place.
#[derive(Debug)]
pub struct EmbeddingRuntime {
    request_timeout: Duration,
    timeout_cooldown: Duration,
    timeout_cooldown_until_ms: AtomicU64,
}

impl EmbeddingRuntime {
    /// Create a new embedding runtime guard.
    #[must_use]
    pub fn new(request_timeout: Duration, timeout_cooldown: Duration) -> Self {
        Self {
            request_timeout,
            timeout_cooldown,
            timeout_cooldown_until_ms: AtomicU64::new(0),
        }
    }

    /// Return configured request timeout.
    #[must_use]
    pub fn request_timeout(&self) -> Duration {
        self.request_timeout
    }

    /// Return configured timeout cooldown duration.
    #[must_use]
    pub fn timeout_cooldown(&self) -> Duration {
        self.timeout_cooldown
    }

    /// Execute one embedding request under timeout/cooldown and repair policy.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryEmbeddingErrorKind::CooldownActive`] when cooldown is active,
    /// [`MemoryEmbeddingErrorKind::Timeout`] when embedding exceeds timeout, and
    /// [`MemoryEmbeddingErrorKind::Unavailable`] when embedding backend returns no vector.
    pub async fn embed_with_source<EmbedFn, EmbedFuture>(
        &self,
        intent: &str,
        expected_dim: usize,
        embed_fn: EmbedFn,
    ) -> Result<(Vec<f32>, &'static str), MemoryEmbeddingErrorKind>
    where
        EmbedFn: FnOnce(&str) -> EmbedFuture,
        EmbedFuture: Future<Output = Option<Vec<f32>>>,
    {
        let cooldown_until = self.timeout_cooldown_until_ms.load(Ordering::Relaxed);
        let now_ms = current_unix_millis();
        if cooldown_until > now_ms {
            tracing::debug!(
                event = "llm.embedding.runtime.cooldown_active",
                cooldown_remaining_ms = cooldown_until.saturating_sub(now_ms),
                cooldown_total_ms = duration_to_u64_millis(self.timeout_cooldown),
                "embedding timeout cooldown active; rejecting request"
            );
            return Err(MemoryEmbeddingErrorKind::CooldownActive);
        }

        match tokio::time::timeout(self.request_timeout, embed_fn(intent)).await {
            Ok(Some(embedded)) => {
                self.timeout_cooldown_until_ms.store(0, Ordering::Relaxed);
                if embedded.len() == expected_dim {
                    return Ok((embedded, EMBEDDING_SOURCE_EMBEDDING));
                }
                let repaired = repair_embedding_dimension(&embedded, expected_dim);
                tracing::warn!(
                    event = "llm.embedding.runtime.dimension_repaired",
                    returned_dim = embedded.len(),
                    expected_dim,
                    repair_strategy = "resample",
                    "embedding dimension mismatch; repaired vector for memory operations"
                );
                Ok((repaired, EMBEDDING_SOURCE_EMBEDDING_REPAIRED))
            }
            Ok(None) => {
                tracing::warn!(
                    event = "llm.embedding.runtime.unavailable",
                    "embedding unavailable; semantic memory operation skipped"
                );
                Err(MemoryEmbeddingErrorKind::Unavailable)
            }
            Err(_) => {
                let cooldown_ms = duration_to_u64_millis(self.timeout_cooldown);
                if cooldown_ms > 0 {
                    self.timeout_cooldown_until_ms.store(
                        current_unix_millis().saturating_add(cooldown_ms),
                        Ordering::Relaxed,
                    );
                }
                tracing::warn!(
                    event = "llm.embedding.runtime.timeout",
                    timeout_ms = self.request_timeout.as_millis(),
                    cooldown_ms,
                    "embedding timed out; semantic memory operation skipped"
                );
                Err(MemoryEmbeddingErrorKind::Timeout)
            }
        }
    }
}

/// Resample an embedding vector to the target dimension.
///
/// This keeps semantic signal when the upstream embedding model dimension drifts
/// from configured memory dimension (for example 1024 -> 384).
#[must_use]
pub fn repair_embedding_dimension(input: &[f32], target_dim: usize) -> Vec<f32> {
    if target_dim == 0 {
        return Vec::new();
    }
    if input.len() == target_dim {
        return input.to_vec();
    }
    if input.is_empty() {
        return vec![0.0; target_dim];
    }
    if input.len() == 1 {
        return vec![input[0]; target_dim];
    }
    if target_dim == 1 {
        let sum = input.iter().copied().sum::<f32>();
        let denom = input.len().to_f32().unwrap_or(1.0);
        return vec![sum / denom];
    }

    let max_input_idx = input.len().saturating_sub(1);
    let source_max = max_input_idx.to_f32().unwrap_or(0.0);
    let target_max = target_dim.saturating_sub(1).to_f32().unwrap_or(1.0);
    let mut repaired = Vec::with_capacity(target_dim);
    for idx in 0..target_dim {
        let idx_f = idx.to_f32().unwrap_or(target_max);
        let position = (idx_f / target_max) * source_max;
        let left = position
            .floor()
            .to_usize()
            .unwrap_or(max_input_idx)
            .min(max_input_idx);
        let right = position
            .ceil()
            .to_usize()
            .unwrap_or(max_input_idx)
            .min(max_input_idx);
        if left == right {
            repaired.push(input[left]);
            continue;
        }
        let mix = position - left.to_f32().unwrap_or(0.0);
        let value = input[left] * (1.0 - mix) + input[right] * mix;
        repaired.push(value);
    }

    normalize(repaired)
}

fn normalize(mut values: Vec<f32>) -> Vec<f32> {
    let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm <= f32::EPSILON {
        return values;
    }
    for value in &mut values {
        *value /= norm;
    }
    values
}

fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

fn duration_to_u64_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
