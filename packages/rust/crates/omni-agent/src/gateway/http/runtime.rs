use std::sync::Arc;

use axum::http::StatusCode;
use tokio::sync::Mutex;
use xiuxian_llm::embedding::backend::{EmbeddingBackendKind, parse_embedding_backend_kind};
use xiuxian_llm::mistral::{ManagedMistralServer, MistralServerConfig, spawn_mistral_server};

use crate::config::{RuntimeSettings, load_runtime_settings};
use crate::embedding::EmbeddingClient;

use super::types::GatewayEmbeddingRuntime;

const DEFAULT_EMBED_TIMEOUT_SECS: u64 = 15;
const DEFAULT_EMBED_UPSTREAM_BASE_URL: &str = "http://127.0.0.1:11434";

pub(super) fn trim_non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn resolve_embed_model(
    requested_model: Option<&str>,
    default_model: Option<&str>,
) -> Result<String, (StatusCode, String)> {
    let configured = trim_non_empty(default_model);
    let requested = trim_non_empty(requested_model);

    if let Some(configured_model) = configured {
        if let Some(requested_model) = requested.as_deref()
            && requested_model != configured_model
        {
            tracing::debug!(
                event = "gateway.embedding.model_override_ignored",
                requested_model,
                configured_model,
                "ignoring request model override; using configured embedding model"
            );
        }
        return Ok(configured_model);
    }

    requested.ok_or((
        StatusCode::BAD_REQUEST,
        "embedding model must be provided (request.model or embedding.model)".to_string(),
    ))
}

pub(super) fn resolve_embed_base_url(
    runtime_settings: &RuntimeSettings,
    backend_hint: Option<&str>,
) -> String {
    let memory_base_url = trim_non_empty(runtime_settings.memory.embedding_base_url.as_deref());
    let litellm_api_base = trim_non_empty(runtime_settings.embedding.litellm_api_base.as_deref());
    let embedding_client_url = trim_non_empty(runtime_settings.embedding.client_url.as_deref());
    let mistral_base_url = trim_non_empty(runtime_settings.mistral.base_url.as_deref());
    let backend_mode = parse_embedding_backend_kind(
        backend_hint
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    );
    let selected = match backend_mode {
        Some(EmbeddingBackendKind::MistralLocal) => mistral_base_url
            .or(memory_base_url)
            .or(embedding_client_url)
            .or(litellm_api_base),
        Some(EmbeddingBackendKind::LiteLlmRs | EmbeddingBackendKind::OpenAiHttp) => {
            litellm_api_base
                .or(memory_base_url)
                .or(embedding_client_url)
        }
        _ => memory_base_url
            .or(embedding_client_url)
            .or(litellm_api_base),
    };
    selected.unwrap_or_else(|| DEFAULT_EMBED_UPSTREAM_BASE_URL.to_string())
}

pub(super) fn build_embedding_runtime() -> GatewayEmbeddingRuntime {
    let runtime_settings = load_runtime_settings();
    build_embedding_runtime_from_settings(&runtime_settings, None, None)
}

pub(super) async fn build_embedding_runtime_for_gateway() -> GatewayEmbeddingRuntime {
    let runtime_settings = load_runtime_settings();
    build_embedding_runtime_for_settings(runtime_settings).await
}

async fn build_embedding_runtime_for_settings(
    runtime_settings: RuntimeSettings,
) -> GatewayEmbeddingRuntime {
    let backend_hint = resolve_backend_hint(&runtime_settings);
    let mut managed_mistral_server: Option<Arc<Mutex<ManagedMistralServer>>> = None;
    let mut base_url_override: Option<String> = None;

    if should_auto_start_mistral(&runtime_settings, backend_hint.as_deref()) {
        let server_config = build_mistral_server_config(&runtime_settings);
        match spawn_mistral_server(server_config).await {
            Ok(server) => {
                tracing::info!(
                    event = "gateway.embedding.mistral.autostart.enabled",
                    pid = server.pid(),
                    base_url = server.base_url(),
                    "mistral server auto-start enabled for gateway embedding runtime"
                );
                base_url_override = Some(server.base_url().to_string());
                managed_mistral_server = Some(Arc::new(Mutex::new(server)));
            }
            Err(error) => {
                tracing::warn!(
                    event = "gateway.embedding.mistral.autostart.failed",
                    error = %error,
                    "failed to auto-start mistral server; continuing with configured embedding upstream"
                );
            }
        }
    }

    build_embedding_runtime_from_settings(
        &runtime_settings,
        managed_mistral_server,
        base_url_override.as_deref(),
    )
}

fn build_embedding_runtime_from_settings(
    runtime_settings: &RuntimeSettings,
    managed_mistral_server: Option<Arc<Mutex<ManagedMistralServer>>>,
    base_url_override: Option<&str>,
) -> GatewayEmbeddingRuntime {
    let backend_hint = resolve_backend_hint(runtime_settings);
    let base_url = resolve_runtime_embed_base_url(
        runtime_settings,
        backend_hint.as_deref(),
        base_url_override,
    );
    let timeout_secs = runtime_settings
        .embedding
        .timeout_secs
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_EMBED_TIMEOUT_SECS);
    let batch_max_size = runtime_settings
        .embedding
        .batch_max_size
        .filter(|value| *value > 0);
    let batch_max_concurrency = runtime_settings
        .embedding
        .batch_max_concurrency
        .filter(|value| *value > 0);
    let default_model = trim_non_empty(runtime_settings.memory.embedding_model.as_deref())
        .or_else(|| trim_non_empty(runtime_settings.embedding.litellm_model.as_deref()))
        .or_else(|| trim_non_empty(runtime_settings.embedding.model.as_deref()));

    tracing::info!(
        event = "gateway.embedding.runtime.initialized",
        backend = backend_hint.as_deref().unwrap_or("default"),
        base_url = %base_url,
        timeout_secs,
        has_default_model = default_model.is_some(),
        batch_max_size = ?batch_max_size,
        batch_max_concurrency = ?batch_max_concurrency,
        "gateway embedding runtime initialized"
    );

    let client = EmbeddingClient::new_with_backend_and_tuning(
        &base_url,
        timeout_secs,
        backend_hint.as_deref(),
        batch_max_size,
        batch_max_concurrency,
    );

    GatewayEmbeddingRuntime {
        client: Arc::new(client),
        default_model,
        managed_mistral_server,
    }
}

pub(super) fn resolve_runtime_embed_base_url(
    runtime_settings: &RuntimeSettings,
    backend_hint: Option<&str>,
    base_url_override: Option<&str>,
) -> String {
    if let Some(override_url) = trim_non_empty(base_url_override) {
        return override_url;
    }
    if is_mistral_backend_hint(backend_hint) {
        return trim_non_empty(runtime_settings.mistral.base_url.as_deref())
            .unwrap_or_else(|| resolve_embed_base_url(runtime_settings, backend_hint));
    }
    resolve_embed_base_url(runtime_settings, backend_hint)
}

fn resolve_backend_hint(runtime_settings: &RuntimeSettings) -> Option<String> {
    trim_non_empty(runtime_settings.memory.embedding_backend.as_deref())
        .or_else(|| trim_non_empty(runtime_settings.embedding.backend.as_deref()))
}

pub(super) fn should_auto_start_mistral(
    runtime_settings: &RuntimeSettings,
    backend_hint: Option<&str>,
) -> bool {
    let enabled = runtime_settings.mistral.enabled.unwrap_or(false);
    let auto_start = runtime_settings.mistral.auto_start.unwrap_or(false);
    enabled && auto_start && is_mistral_backend_hint(backend_hint)
}

fn is_mistral_backend_hint(backend_hint: Option<&str>) -> bool {
    let normalized = backend_hint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| parse_embedding_backend_kind(Some(value)));

    matches!(normalized, Some(EmbeddingBackendKind::MistralLocal))
}

pub(super) fn build_mistral_server_config(
    runtime_settings: &RuntimeSettings,
) -> MistralServerConfig {
    let mut config = MistralServerConfig::from_env();

    if let Some(command) = trim_non_empty(runtime_settings.mistral.command.as_deref()) {
        config.command = command;
    }
    if let Some(args) = runtime_settings.mistral.args.as_ref() {
        let normalized_args = args
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if !normalized_args.is_empty() {
            config.args = normalized_args;
        }
    }
    if let Some(base_url) = trim_non_empty(runtime_settings.mistral.base_url.as_deref()) {
        config.base_url = base_url;
    }
    if let Some(timeout_secs) = runtime_settings.mistral.startup_timeout_secs {
        config.startup_timeout_secs = timeout_secs.max(1);
    }
    if let Some(timeout_ms) = runtime_settings.mistral.probe_timeout_ms {
        config.probe_timeout_ms = timeout_ms.max(1);
    }
    if let Some(interval_ms) = runtime_settings.mistral.probe_interval_ms {
        config.probe_interval_ms = interval_ms.max(1);
    }

    config
}

#[cfg(test)]
#[path = "../../../tests/gateway/http/runtime.rs"]
mod tests;
