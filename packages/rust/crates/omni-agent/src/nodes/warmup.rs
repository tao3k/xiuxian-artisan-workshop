use std::time::Instant;

use anyhow::{Result, anyhow};
use omni_agent::{EmbeddingClient, RuntimeSettings};
use xiuxian_llm::embedding::backend::{EmbeddingBackendKind, parse_embedding_backend_kind};

use crate::resolve::{parse_positive_u64_from_env, parse_positive_usize_from_env};

const DEFAULT_MEMORY_EMBED_BASE_URL: &str = "http://127.0.0.1:3002";
const DEFAULT_EMBED_TIMEOUT_SECS: u64 = 15;
const MISTRAL_SDK_INPROC_LABEL: &str = "inproc://mistral-sdk";

#[derive(Debug, Clone, Default)]
struct WarmupEnvOverrides {
    memory_embedding_backend: Option<String>,
    embedding_backend: Option<String>,
    llm_backend: Option<String>,
    memory_embedding_model: Option<String>,
    embedding_model: Option<String>,
    memory_embedding_base_url: Option<String>,
    embedding_base_url: Option<String>,
    embed_timeout_secs: Option<u64>,
    memory_embed_batch_max_size: Option<usize>,
    embed_batch_max_size: Option<usize>,
    memory_embed_batch_max_concurrency: Option<usize>,
    embed_batch_max_concurrency: Option<usize>,
    mistral_sdk_hf_cache_path: Option<String>,
    mistral_sdk_hf_revision: Option<String>,
}

impl WarmupEnvOverrides {
    fn from_process_env() -> Self {
        Self {
            memory_embedding_backend: non_empty_env("OMNI_AGENT_MEMORY_EMBEDDING_BACKEND"),
            embedding_backend: non_empty_env("OMNI_AGENT_EMBED_BACKEND"),
            llm_backend: non_empty_env("OMNI_AGENT_LLM_BACKEND"),
            memory_embedding_model: non_empty_env("OMNI_AGENT_MEMORY_EMBEDDING_MODEL"),
            embedding_model: non_empty_env("OMNI_AGENT_EMBED_MODEL"),
            memory_embedding_base_url: non_empty_env("OMNI_AGENT_MEMORY_EMBEDDING_BASE_URL"),
            embedding_base_url: non_empty_env("OMNI_AGENT_EMBED_BASE_URL"),
            embed_timeout_secs: parse_positive_u64_from_env("OMNI_AGENT_EMBED_TIMEOUT_SECS"),
            memory_embed_batch_max_size: parse_positive_usize_from_env(
                "OMNI_AGENT_MEMORY_EMBED_BATCH_MAX_SIZE",
            ),
            embed_batch_max_size: parse_positive_usize_from_env("OMNI_AGENT_EMBED_BATCH_MAX_SIZE"),
            memory_embed_batch_max_concurrency: parse_positive_usize_from_env(
                "OMNI_AGENT_MEMORY_EMBED_BATCH_MAX_CONCURRENCY",
            ),
            embed_batch_max_concurrency: parse_positive_usize_from_env(
                "OMNI_AGENT_EMBED_BATCH_MAX_CONCURRENCY",
            ),
            mistral_sdk_hf_cache_path: non_empty_env("OMNI_AGENT_MISTRAL_SDK_HF_CACHE_PATH"),
            mistral_sdk_hf_revision: non_empty_env("OMNI_AGENT_MISTRAL_SDK_HF_REVISION"),
        }
    }
}

#[derive(Debug, Clone)]
struct WarmupOptions {
    backend_hint: Option<String>,
    model: Option<String>,
    base_url: String,
    timeout_secs: u64,
    batch_max_size: Option<usize>,
    batch_max_concurrency: Option<usize>,
    mistral_sdk_hf_cache_path: Option<String>,
    mistral_sdk_hf_revision: Option<String>,
}

pub(crate) async fn run_embedding_warmup(
    runtime_settings: &RuntimeSettings,
    text: String,
    model_override: Option<String>,
    mistral_sdk_only: bool,
) -> Result<()> {
    let env = WarmupEnvOverrides::from_process_env();
    let options = resolve_warmup_options(runtime_settings, &env, model_override.as_deref());
    let backend_kind = parse_embedding_backend_kind(options.backend_hint.as_deref());
    let display_base_url = if matches!(backend_kind, Some(EmbeddingBackendKind::MistralSdk)) {
        MISTRAL_SDK_INPROC_LABEL
    } else {
        options.base_url.as_str()
    };
    if mistral_sdk_only && !matches!(backend_kind, Some(EmbeddingBackendKind::MistralSdk)) {
        println!(
            "Embedding warmup skipped: effective backend='{}' is not mistral_sdk",
            options.backend_hint.as_deref().unwrap_or("auto")
        );
        return Ok(());
    }
    println!(
        "Embedding warmup starting: backend='{}' model='{}' timeout_secs={} base_url='{}'",
        options.backend_hint.as_deref().unwrap_or("auto"),
        options.model.as_deref().unwrap_or("<default>"),
        options.timeout_secs,
        display_base_url
    );
    if matches!(backend_kind, Some(EmbeddingBackendKind::MistralSdk)) {
        println!(
            "Mistral SDK cache: hf_cache_path='{}' hf_revision='{}'",
            options
                .mistral_sdk_hf_cache_path
                .as_deref()
                .unwrap_or("<default>"),
            options
                .mistral_sdk_hf_revision
                .as_deref()
                .unwrap_or("<default>")
        );
        println!("Mistral SDK transport: in-process Rust runtime (HTTP base_url ignored).");
    }

    let client = EmbeddingClient::new_with_backend_and_tuning(
        options.base_url.as_str(),
        options.timeout_secs,
        options.backend_hint.as_deref(),
        options.batch_max_size,
        options.batch_max_concurrency,
    );
    let started = Instant::now();
    let maybe_vector = client
        .embed_with_model(text.as_str(), options.model.as_deref())
        .await;
    let elapsed_ms = started.elapsed().as_millis();
    match maybe_vector {
        Some(vector) => {
            println!(
                "Embedding warmup succeeded: dim={} elapsed_ms={elapsed_ms}",
                vector.len()
            );
            Ok(())
        }
        None => Err(anyhow!(
            "embedding warmup failed: backend='{}' model='{}' base_url='{}'",
            options.backend_hint.as_deref().unwrap_or("auto"),
            options.model.as_deref().unwrap_or("<default>"),
            options.base_url
        )),
    }
}

fn resolve_warmup_options(
    runtime_settings: &RuntimeSettings,
    env: &WarmupEnvOverrides,
    model_override: Option<&str>,
) -> WarmupOptions {
    let backend_hint = first_non_empty([
        env.memory_embedding_backend.clone(),
        trim_non_empty(runtime_settings.memory.embedding_backend.as_deref()),
        env.embedding_backend.clone(),
        trim_non_empty(runtime_settings.embedding.backend.as_deref()),
        env.llm_backend.clone(),
        trim_non_empty(runtime_settings.agent.llm_backend.as_deref()),
    ]);

    let model = trim_non_empty(model_override)
        .or_else(|| env.memory_embedding_model.clone())
        .or_else(|| trim_non_empty(runtime_settings.memory.embedding_model.as_deref()))
        .or_else(|| env.embedding_model.clone())
        .or_else(|| trim_non_empty(runtime_settings.embedding.litellm_model.as_deref()))
        .or_else(|| trim_non_empty(runtime_settings.embedding.model.as_deref()));

    let base_url = first_non_empty([
        env.memory_embedding_base_url.clone(),
        trim_non_empty(runtime_settings.memory.embedding_base_url.as_deref()),
        env.embedding_base_url.clone(),
        trim_non_empty(runtime_settings.embedding.client_url.as_deref()),
        trim_non_empty(runtime_settings.embedding.litellm_api_base.as_deref()),
        trim_non_empty(runtime_settings.mistral.base_url.as_deref()),
    ])
    .unwrap_or_else(|| DEFAULT_MEMORY_EMBED_BASE_URL.to_string());

    let timeout_secs = env
        .embed_timeout_secs
        .or(runtime_settings.embedding.timeout_secs)
        .unwrap_or(DEFAULT_EMBED_TIMEOUT_SECS);

    let batch_max_size = env
        .memory_embed_batch_max_size
        .or(env.embed_batch_max_size)
        .or(runtime_settings
            .embedding
            .batch_max_size
            .filter(|value| *value > 0));

    let batch_max_concurrency = env
        .memory_embed_batch_max_concurrency
        .or(env.embed_batch_max_concurrency)
        .or(runtime_settings
            .embedding
            .batch_max_concurrency
            .filter(|value| *value > 0));

    let mistral_sdk_hf_cache_path = env
        .mistral_sdk_hf_cache_path
        .clone()
        .or_else(|| trim_non_empty(runtime_settings.mistral.sdk_hf_cache_path.as_deref()));

    let mistral_sdk_hf_revision = env
        .mistral_sdk_hf_revision
        .clone()
        .or_else(|| trim_non_empty(runtime_settings.mistral.sdk_hf_revision.as_deref()));

    WarmupOptions {
        backend_hint,
        model,
        base_url,
        timeout_secs,
        batch_max_size,
        batch_max_concurrency,
        mistral_sdk_hf_cache_path,
        mistral_sdk_hf_revision,
    }
}

fn first_non_empty<const N: usize>(values: [Option<String>; N]) -> Option<String> {
    values.into_iter().flatten().next()
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .and_then(|raw| trim_non_empty(Some(raw.as_str())))
}

fn trim_non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|candidate| !candidate.is_empty())
        .map(ToString::to_string)
}
