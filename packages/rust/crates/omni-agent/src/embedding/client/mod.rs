mod backend_dispatch;
mod batch;
mod chunk_dispatch;
mod init;
mod support;

use std::sync::Arc;

use tokio::sync::Semaphore;

use super::backend::EmbeddingBackendMode;
use super::cache::EmbeddingCache;

const DEFAULT_EMBED_CACHE_TTL_SECS: u64 = 900;
const MAX_EMBED_CACHE_TTL_SECS: u64 = 86_400;
const DEFAULT_EMBED_CACHE_MAX_ENTRIES: usize = 4_096;
const MAX_EMBED_CACHE_MAX_ENTRIES: usize = 65_536;
const DEFAULT_EMBED_BATCH_MAX_SIZE: usize = 128;
const MAX_EMBED_BATCH_MAX_SIZE: usize = 8_192;
const DEFAULT_EMBED_BATCH_MAX_CONCURRENCY: usize = 1;
const MAX_EMBED_BATCH_MAX_CONCURRENCY: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmbeddingInFlightSnapshot {
    pub max_in_flight: usize,
    pub available_permits: usize,
    pub in_flight: usize,
    pub saturation_pct: u8,
}

/// Embedding client runtime.
pub struct EmbeddingClient {
    client: reqwest::Client,
    base_url: String,
    mcp_url: Option<String>,
    cache: EmbeddingCache,
    backend_mode: EmbeddingBackendMode,
    backend_source: &'static str,
    #[cfg(feature = "agent-provider-litellm")]
    timeout_secs: u64,
    max_in_flight: Option<usize>,
    in_flight_gate: Option<Arc<Semaphore>>,
    batch_max_size: usize,
    batch_max_concurrency: usize,
    default_model: Option<String>,
    mistral_sdk_hf_cache_path: Option<String>,
    mistral_sdk_hf_revision: Option<String>,
    mistral_sdk_max_num_seqs: Option<usize>,
    #[cfg(feature = "agent-provider-litellm")]
    litellm_api_key: Option<String>,
}

#[derive(Clone)]
struct EmbeddingDispatchRuntime {
    client: reqwest::Client,
    base_url: String,
    mcp_url: Option<String>,
    backend_mode: EmbeddingBackendMode,
    backend_source: &'static str,
    #[cfg(feature = "agent-provider-litellm")]
    timeout_secs: u64,
    max_in_flight: Option<usize>,
    in_flight_gate: Option<Arc<Semaphore>>,
    mistral_sdk_hf_cache_path: Option<String>,
    mistral_sdk_hf_revision: Option<String>,
    mistral_sdk_max_num_seqs: Option<usize>,
    #[cfg(feature = "agent-provider-litellm")]
    litellm_api_key: Option<String>,
}

impl EmbeddingClient {
    /// Return current in-flight permit usage snapshot when throttling is enabled.
    #[must_use]
    pub fn in_flight_snapshot(&self) -> Option<EmbeddingInFlightSnapshot> {
        let max_in_flight = self.max_in_flight?;
        let available_permits = self.in_flight_gate.as_ref().map_or(max_in_flight, |gate| {
            gate.available_permits().min(max_in_flight)
        });
        let in_flight = max_in_flight.saturating_sub(available_permits);
        let saturation_pct = compute_saturation_pct(in_flight, max_in_flight);
        Some(EmbeddingInFlightSnapshot {
            max_in_flight,
            available_permits,
            in_flight,
            saturation_pct,
        })
    }
}

fn compute_saturation_pct(in_flight: usize, max_in_flight: usize) -> u8 {
    if max_in_flight == 0 {
        return 0;
    }
    let ratio = in_flight.saturating_mul(100) / max_in_flight;
    u8::try_from(ratio).unwrap_or(100).min(100)
}
