use super::super::memory_state::{MemoryStateBackend, MemoryStateLoadStatus};
use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use crate::config::{AgentConfig, RuntimeSettings};
use crate::embedding::EmbeddingClient;
use crate::observability::SessionEvent;
use crate::session::SessionStore;
use anyhow::{Context, Result};
use omni_memory::{EpisodeStore, StoreConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};
use xiuxian_llm::embedding::backend::{EmbeddingBackendKind, parse_embedding_backend_kind};
use xiuxian_llm::embedding::runtime::{
    DEFAULT_MEMORY_EMBED_TIMEOUT, DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN, EmbeddingRuntime,
    MAX_MEMORY_EMBED_COOLDOWN_MS, MAX_MEMORY_EMBED_TIMEOUT_MS, MIN_MEMORY_EMBED_TIMEOUT_MS,
};
use xiuxian_macros::env_non_empty;

const DEFAULT_MEMORY_EMBED_BASE_URL: &str = "http://localhost:3002";
const MISTRAL_SDK_INPROC_LABEL: &str = "inproc://mistral-sdk";

pub(super) struct MemoryRuntimeBuild {
    pub(super) memory_store: Option<Arc<EpisodeStore>>,
    pub(super) memory_state_backend: Option<Arc<MemoryStateBackend>>,
    pub(super) memory_state_load_status: MemoryStateLoadStatus,
    pub(super) embedding_client: Option<EmbeddingClient>,
    pub(super) embedding_runtime: Option<Arc<EmbeddingRuntime>>,
    pub(super) memory_stream_consumer_task: Option<tokio::task::JoinHandle<()>>,
}

pub(super) fn build_memory_runtime(
    config: &AgentConfig,
    session: &SessionStore,
    runtime_settings: &RuntimeSettings,
    mounts: &mut ServiceMountCatalog,
) -> Result<MemoryRuntimeBuild> {
    let (memory_store, memory_state_backend, memory_state_load_status) =
        init_memory_backends(config, mounts)?;

    let embedding_client = config.memory.as_ref().map(|memory_cfg| {
        let embed_timeout_secs = memory_cfg
            .embedding_timeout_ms
            .map_or(15, timeout_ms_to_timeout_secs);
        let base_url = resolve_memory_embed_base_url(memory_cfg, runtime_settings);
        EmbeddingClient::new_with_backend_and_tuning(
            &base_url,
            embed_timeout_secs,
            memory_cfg.embedding_backend.as_deref(),
            memory_cfg.embedding_batch_max_size,
            memory_cfg.embedding_batch_max_concurrency,
        )
    });
    if let Some(memory_cfg) = config.memory.as_ref() {
        let base_url = resolve_memory_embed_base_url(memory_cfg, runtime_settings);
        mounts.mounted(
            "memory.embedding_client",
            "memory",
            ServiceMountMeta::default()
                .endpoint(base_url)
                .storage(memory_cfg.path.clone())
                .detail(format!(
                    "backend={}",
                    memory_cfg.embedding_backend.as_deref().unwrap_or("default")
                )),
        );
    } else {
        mounts.skipped(
            "memory.embedding_client",
            "memory",
            ServiceMountMeta::default().detail("memory config disabled"),
        );
    }

    let memory_stream_consumer_task = config.memory.as_ref().and_then(|memory_cfg| {
        super::super::memory_stream_consumer::spawn_memory_stream_consumer(
            memory_cfg,
            session.redis_runtime_snapshot(),
        )
    });
    if memory_stream_consumer_task.is_some() {
        mounts.mounted(
            "memory.stream_consumer",
            "memory",
            ServiceMountMeta::default().detail("valkey stream consumer started"),
        );
    } else {
        mounts.skipped(
            "memory.stream_consumer",
            "memory",
            ServiceMountMeta::default().detail("memory stream consumer disabled"),
        );
    }

    let memory_embed_timeout_default_ms = config
        .memory
        .as_ref()
        .and_then(|memory_cfg| memory_cfg.embedding_timeout_ms)
        .unwrap_or_else(|| duration_to_u64_millis(DEFAULT_MEMORY_EMBED_TIMEOUT));
    let memory_embed_timeout = duration_from_env_ms(
        "OMNI_AGENT_MEMORY_EMBED_TIMEOUT_MS",
        memory_embed_timeout_default_ms,
        MIN_MEMORY_EMBED_TIMEOUT_MS,
        MAX_MEMORY_EMBED_TIMEOUT_MS,
    );
    let memory_embed_timeout_cooldown_default_ms = config
        .memory
        .as_ref()
        .and_then(|memory_cfg| memory_cfg.embedding_timeout_cooldown_ms)
        .unwrap_or_else(|| duration_to_u64_millis(DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN));
    let memory_embed_timeout_cooldown = duration_from_env_ms(
        "OMNI_AGENT_MEMORY_EMBED_TIMEOUT_COOLDOWN_MS",
        memory_embed_timeout_cooldown_default_ms,
        0,
        MAX_MEMORY_EMBED_COOLDOWN_MS,
    );
    let embedding_runtime = embedding_client.as_ref().map(|_| {
        Arc::new(EmbeddingRuntime::new(
            memory_embed_timeout,
            memory_embed_timeout_cooldown,
        ))
    });

    Ok(MemoryRuntimeBuild {
        memory_store,
        memory_state_backend,
        memory_state_load_status,
        embedding_client,
        embedding_runtime,
        memory_stream_consumer_task,
    })
}

type MemoryBackendInit = (
    Option<Arc<EpisodeStore>>,
    Option<Arc<MemoryStateBackend>>,
    MemoryStateLoadStatus,
);

fn init_memory_backends(
    config: &AgentConfig,
    mounts: &mut ServiceMountCatalog,
) -> Result<MemoryBackendInit> {
    let Some(memory_cfg) = config.memory.as_ref() else {
        mounts.skipped(
            "memory.state_backend",
            "memory",
            ServiceMountMeta::default().detail("memory config disabled"),
        );
        return Ok((None, None, MemoryStateLoadStatus::NotConfigured));
    };

    let backend = match MemoryStateBackend::from_config(memory_cfg) {
        Ok(backend) => backend,
        Err(error) => {
            mounts.failed(
                "memory.state_backend",
                "memory",
                ServiceMountMeta::default()
                    .storage(memory_cfg.path.clone())
                    .detail(format!("backend init failed: {error}")),
            );
            return Err(error);
        }
    };

    let store = EpisodeStore::new(StoreConfig {
        path: memory_cfg.path.clone(),
        embedding_dim: memory_cfg.embedding_dim,
        table_name: memory_cfg.table_name.clone(),
    });
    let load_started = Instant::now();
    let load_status = match backend.load(&store) {
        Ok(()) => {
            tracing::debug!(
                event = SessionEvent::MemoryStateLoadSucceeded.as_str(),
                backend = backend.backend_name(),
                strict_startup = backend.strict_startup(),
                episodes = store.len(),
                q_values = store.q_table.len(),
                duration_ms = load_started.elapsed().as_millis(),
                "memory state loaded from persistence backend"
            );
            mounts.mounted(
                "memory.state_backend",
                "memory",
                ServiceMountMeta::default()
                    .storage(memory_cfg.path.clone())
                    .detail(format!(
                        "backend={} strict_startup={} load_status={}",
                        backend.backend_name(),
                        backend.strict_startup(),
                        MemoryStateLoadStatus::Loaded.as_str()
                    )),
            );
            MemoryStateLoadStatus::Loaded
        }
        Err(error) => {
            if backend.strict_startup() {
                mounts.failed(
                    "memory.state_backend",
                    "memory",
                    ServiceMountMeta::default()
                        .storage(memory_cfg.path.clone())
                        .detail(format!(
                            "strict startup load failed (backend={}): {error}",
                            backend.backend_name()
                        )),
                );
                return Err(error).context("strict valkey memory backend failed during startup");
            }
            mounts.failed(
                "memory.state_backend",
                "memory",
                ServiceMountMeta::default()
                    .storage(memory_cfg.path.clone())
                    .detail(format!(
                        "load failed but continuing (backend={} strict_startup=false): {error}",
                        backend.backend_name()
                    )),
            );
            MemoryStateLoadStatus::LoadFailedContinue
        }
    };

    Ok((Some(Arc::new(store)), Some(Arc::new(backend)), load_status))
}

fn duration_to_u64_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn duration_from_env_ms(name: &str, default_ms: u64, min_ms: u64, max_ms: u64) -> Duration {
    let parsed = env_non_empty!(name)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default_ms);
    let capped = parsed.min(max_ms);
    let sanitized = if capped < min_ms { min_ms } else { capped };
    Duration::from_millis(sanitized)
}

fn trimmed_non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn resolve_memory_embed_base_url(
    memory_cfg: &crate::config::MemoryConfig,
    runtime_settings: &RuntimeSettings,
) -> String {
    let backend_hint = trimmed_non_empty(memory_cfg.embedding_backend.as_deref())
        .or_else(|| trimmed_non_empty(runtime_settings.memory.embedding_backend.as_deref()))
        .or_else(|| trimmed_non_empty(runtime_settings.embedding.backend.as_deref()));
    if matches!(
        parse_embedding_backend_kind(backend_hint.as_deref()),
        Some(EmbeddingBackendKind::MistralSdk)
    ) {
        return MISTRAL_SDK_INPROC_LABEL.to_string();
    }

    trimmed_non_empty(memory_cfg.embedding_base_url.as_deref())
        .or_else(|| {
            trimmed_non_empty(env_non_empty!("OMNI_AGENT_MEMORY_EMBEDDING_BASE_URL").as_deref())
        })
        .or_else(|| trimmed_non_empty(env_non_empty!("OMNI_AGENT_EMBED_BASE_URL").as_deref()))
        .or_else(|| trimmed_non_empty(runtime_settings.memory.embedding_base_url.as_deref()))
        .or_else(|| trimmed_non_empty(runtime_settings.embedding.client_url.as_deref()))
        .or_else(|| trimmed_non_empty(runtime_settings.embedding.litellm_api_base.as_deref()))
        .or_else(|| trimmed_non_empty(runtime_settings.mistral.base_url.as_deref()))
        .unwrap_or_else(|| DEFAULT_MEMORY_EMBED_BASE_URL.to_string())
}

fn timeout_ms_to_timeout_secs(timeout_ms: u64) -> u64 {
    let secs = timeout_ms.saturating_add(999) / 1_000;
    secs.max(1)
}
