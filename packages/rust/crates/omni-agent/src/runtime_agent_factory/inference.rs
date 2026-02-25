use anyhow::{Result, anyhow};
use omni_agent::{LITELLM_DEFAULT_URL, McpServerEntry, RuntimeSettings};
use xiuxian_llm::embedding::backend::parse_embedding_backend_kind;
use xiuxian_llm::llm::backend::{LlmBackendKind, parse_llm_backend_kind};

use crate::resolve::parse_bool_from_env;

use super::shared::non_empty_env;
use super::types::RuntimeEmbeddingBackendMode;

fn normalize_inference_url(raw: &str) -> String {
    let u = raw.trim_end_matches('/');
    if u.ends_with("/v1/chat/completions") || u.ends_with("/chat/completions") {
        u.to_string()
    } else if u.ends_with("/v1") {
        format!("{u}/chat/completions")
    } else {
        format!("{}/v1/chat/completions", u.trim_end_matches('/'))
    }
}

pub(super) fn resolve_inference_url(
    litellm_proxy_url: Option<&str>,
    agent_inference_url: Option<&str>,
) -> String {
    let raw = litellm_proxy_url
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            agent_inference_url
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or(LITELLM_DEFAULT_URL);
    normalize_inference_url(raw)
}

fn resolve_inference_url_with_settings(
    litellm_proxy_url: Option<&str>,
    agent_inference_url: Option<&str>,
    runtime_settings: &RuntimeSettings,
) -> String {
    if litellm_proxy_url
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        || agent_inference_url
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
    {
        return resolve_inference_url(litellm_proxy_url, agent_inference_url);
    }

    if let Some(base_url) = runtime_settings
        .inference
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return normalize_inference_url(base_url);
    }

    if matches!(
        resolve_runtime_llm_backend_mode(runtime_settings),
        Some(LlmBackendKind::MistralLocal)
    ) && let Some(base_url) = runtime_settings
        .mistral
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return normalize_inference_url(base_url);
    }

    if runtime_settings
        .inference
        .provider
        .as_deref()
        .map(str::trim)
        .is_some_and(|provider| provider.eq_ignore_ascii_case("minimax"))
    {
        return normalize_inference_url("https://api.minimax.io/v1");
    }

    resolve_inference_url(litellm_proxy_url, agent_inference_url)
}

fn parse_llm_backend_mode(raw: Option<&str>) -> Option<LlmBackendKind> {
    let trimmed = raw.map(str::trim).filter(|value| !value.is_empty());
    let parsed = parse_llm_backend_kind(trimmed);
    if parsed.is_none()
        && let Some(value) = trimmed
    {
        tracing::warn!(
            invalid_value = %value,
            "invalid llm backend mode in runtime settings; ignoring override"
        );
    }
    parsed
}

fn resolve_runtime_llm_backend_mode(runtime_settings: &RuntimeSettings) -> Option<LlmBackendKind> {
    parse_llm_backend_mode(non_empty_env("OMNI_AGENT_LLM_BACKEND").as_deref())
        .or_else(|| parse_llm_backend_mode(runtime_settings.agent.llm_backend.as_deref()))
}

pub(super) fn parse_embedding_backend_mode(
    raw: Option<&str>,
) -> Option<RuntimeEmbeddingBackendMode> {
    let trimmed = raw.map(str::trim).filter(|value| !value.is_empty());
    let parsed = parse_embedding_backend_kind(trimmed);
    if parsed.is_none()
        && let Some(value) = trimmed
    {
        tracing::warn!(
            invalid_value = %value,
            "invalid embedding backend mode in runtime settings; defaulting to http"
        );
    }
    parsed
}

pub(super) fn resolve_runtime_embedding_backend_mode(
    runtime_settings: &RuntimeSettings,
) -> RuntimeEmbeddingBackendMode {
    parse_embedding_backend_mode(non_empty_env("OMNI_AGENT_MEMORY_EMBEDDING_BACKEND").as_deref())
        .or_else(|| {
            parse_embedding_backend_mode(runtime_settings.memory.embedding_backend.as_deref())
        })
        .or_else(|| {
            parse_embedding_backend_mode(non_empty_env("OMNI_AGENT_EMBED_BACKEND").as_deref())
        })
        .or_else(|| parse_embedding_backend_mode(runtime_settings.embedding.backend.as_deref()))
        .or_else(|| {
            parse_embedding_backend_mode(non_empty_env("OMNI_AGENT_LLM_BACKEND").as_deref())
        })
        .or_else(|| parse_embedding_backend_mode(runtime_settings.agent.llm_backend.as_deref()))
        .unwrap_or(default_runtime_embedding_backend_mode())
}

fn default_runtime_embedding_backend_mode() -> RuntimeEmbeddingBackendMode {
    #[cfg(feature = "agent-provider-litellm")]
    {
        RuntimeEmbeddingBackendMode::LiteLlmRs
    }
    #[cfg(not(feature = "agent-provider-litellm"))]
    {
        RuntimeEmbeddingBackendMode::Http
    }
}

pub(super) fn resolve_runtime_embedding_base_url(
    runtime_settings: &RuntimeSettings,
    backend_mode: RuntimeEmbeddingBackendMode,
) -> Option<String> {
    let trim_non_empty = |value: Option<&str>| {
        value
            .map(str::trim)
            .filter(|candidate| !candidate.is_empty())
            .map(ToString::to_string)
    };
    let memory_base_url = trim_non_empty(runtime_settings.memory.embedding_base_url.as_deref());
    let litellm_api_base = trim_non_empty(runtime_settings.embedding.litellm_api_base.as_deref());
    let embedding_client_url = trim_non_empty(runtime_settings.embedding.client_url.as_deref());
    let mistral_base_url = trim_non_empty(runtime_settings.mistral.base_url.as_deref());
    match backend_mode {
        RuntimeEmbeddingBackendMode::Http => memory_base_url
            .or(embedding_client_url)
            .or(litellm_api_base),
        RuntimeEmbeddingBackendMode::MistralLocal => mistral_base_url
            .or(memory_base_url)
            .or(embedding_client_url)
            .or(litellm_api_base),
        RuntimeEmbeddingBackendMode::OpenAiHttp | RuntimeEmbeddingBackendMode::LiteLlmRs => {
            litellm_api_base
                .or(memory_base_url)
                .or(embedding_client_url)
        }
    }
}

fn endpoint_origin(url: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    let port = parsed.port_or_known_default()?;
    Some(format!("{}://{}:{}", parsed.scheme(), host, port))
}

pub(super) fn validate_inference_url_origin(
    inference_url: &str,
    mcp_servers: &[McpServerEntry],
    allow_shared_origin: bool,
) -> Result<()> {
    if allow_shared_origin {
        return Ok(());
    }
    let Some(inference_origin) = endpoint_origin(inference_url) else {
        return Ok(());
    };
    let conflicts: Vec<String> = mcp_servers
        .iter()
        .filter_map(|entry| {
            let url = entry.url.as_deref()?;
            let origin = endpoint_origin(url)?;
            if origin == inference_origin {
                Some(format!("{}={}", entry.name, url))
            } else {
                None
            }
        })
        .collect();
    if conflicts.is_empty() {
        return Ok(());
    }
    Err(anyhow!(
        "invalid inference URL: {} shares origin {} with MCP server(s): {}. \
Use a dedicated LLM endpoint via LITELLM_PROXY_URL or OMNI_AGENT_INFERENCE_URL \
(for example {}). If you intentionally run MCP and inference on one origin, set \
OMNI_AGENT_ALLOW_INFERENCE_MCP_SHARED_ORIGIN=true.",
        inference_url,
        inference_origin,
        conflicts.join(", "),
        LITELLM_DEFAULT_URL,
    ))
}

pub(super) fn resolve_runtime_inference_url(
    runtime_settings: &RuntimeSettings,
    mcp_servers: &[McpServerEntry],
) -> Result<String> {
    let litellm_proxy_url = non_empty_env("LITELLM_PROXY_URL");
    let agent_inference_url = non_empty_env("OMNI_AGENT_INFERENCE_URL");
    let inference_url = resolve_inference_url_with_settings(
        litellm_proxy_url.as_deref(),
        agent_inference_url.as_deref(),
        runtime_settings,
    );
    let allow_shared_origin =
        parse_bool_from_env("OMNI_AGENT_ALLOW_INFERENCE_MCP_SHARED_ORIGIN").unwrap_or(false);
    validate_inference_url_origin(&inference_url, mcp_servers, allow_shared_origin)?;
    Ok(inference_url)
}

pub(super) fn resolve_runtime_model(runtime_settings: &RuntimeSettings) -> String {
    non_empty_env("OMNI_AGENT_MODEL")
        .or_else(|| {
            runtime_settings
                .inference
                .model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
        .unwrap_or_default()
}
