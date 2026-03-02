//! Runtime inference URL and backend resolution tests.

use super::*;

fn mcp_server(url: &str) -> McpServerEntry {
    McpServerEntry {
        name: "local-mcp".to_string(),
        url: Some(url.to_string()),
        command: None,
        args: None,
    }
}

#[test]
fn resolve_inference_url_prefers_default_litellm_when_env_absent() {
    let resolved = resolve_inference_url(None, None);
    assert_eq!(resolved, LITELLM_DEFAULT_URL);
}

#[test]
fn resolve_inference_url_normalizes_completion_path() {
    let resolved = resolve_inference_url(Some("http://127.0.0.1:4000"), None);
    assert_eq!(resolved, "http://127.0.0.1:4000/v1/chat/completions");
}

#[test]
fn resolve_inference_url_does_not_duplicate_v1_path() {
    let resolved = resolve_inference_url(Some("https://api.minimax.io/v1"), None);
    assert_eq!(resolved, "https://api.minimax.io/v1/chat/completions");
}

#[test]
fn resolve_runtime_inference_url_uses_minimax_provider_default_when_configured() {
    let mut settings = RuntimeSettings::default();
    settings.inference.provider = Some("minimax".to_string());

    let resolved = match resolve_runtime_inference_url(&settings, &[]) {
        Ok(resolved) => resolved,
        Err(error) => panic!("minimax provider default should resolve: {error}"),
    };
    assert_eq!(resolved, "https://api.minimax.io/v1/chat/completions");
}

#[test]
fn validate_inference_url_origin_rejects_same_origin_as_mcp_by_default() {
    let servers = vec![mcp_server("http://127.0.0.1:3002/sse")];
    let err = match validate_inference_url_origin(
        "http://127.0.0.1:3002/v1/chat/completions",
        &servers,
        false,
    ) {
        Ok(()) => panic!("shared origin should be rejected by default"),
        Err(err) => err,
    };
    let message = format!("{err:#}");
    assert!(message.contains("invalid inference URL"));
    assert!(message.contains("OMNI_AGENT_ALLOW_INFERENCE_MCP_SHARED_ORIGIN=true"));
}

#[test]
fn validate_inference_url_origin_allows_distinct_origin() {
    let servers = vec![mcp_server("http://127.0.0.1:3002/sse")];
    if let Err(error) =
        validate_inference_url_origin("http://127.0.0.1:4000/v1/chat/completions", &servers, false)
    {
        panic!("distinct origin should be valid: {error}");
    }
}

#[test]
fn validate_inference_url_origin_allows_shared_origin_when_opted_in() {
    let servers = vec![mcp_server("http://127.0.0.1:3002/sse")];
    if let Err(error) =
        validate_inference_url_origin("http://127.0.0.1:3002/v1/chat/completions", &servers, true)
    {
        panic!("opt-in should allow shared origin: {error}");
    }
}

#[test]
fn parse_embedding_backend_mode_supports_litellm_aliases() {
    assert_eq!(
        parse_embedding_backend_mode(Some("litellm_rs")),
        Some(RuntimeEmbeddingBackendMode::LiteLlmRs)
    );
    assert_eq!(
        parse_embedding_backend_mode(Some("litellm-rs")),
        Some(RuntimeEmbeddingBackendMode::LiteLlmRs)
    );
    assert_eq!(
        parse_embedding_backend_mode(Some("provider")),
        Some(RuntimeEmbeddingBackendMode::LiteLlmRs)
    );
}

#[test]
fn parse_embedding_backend_mode_supports_mistral_sdk_aliases() {
    assert_eq!(
        parse_embedding_backend_mode(Some("mistral_sdk")),
        Some(RuntimeEmbeddingBackendMode::MistralSdk)
    );
    assert_eq!(
        parse_embedding_backend_mode(Some("mistral-inproc")),
        Some(RuntimeEmbeddingBackendMode::MistralSdk)
    );
    assert_eq!(
        parse_embedding_backend_mode(Some("openai_http")),
        Some(RuntimeEmbeddingBackendMode::OpenAiHttp)
    );
}

#[test]
fn resolve_runtime_embedding_base_url_prefers_http_client_url_for_http_backend() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.client_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());

    let resolved = resolve_runtime_embedding_base_url(&settings, RuntimeEmbeddingBackendMode::Http);
    assert_eq!(resolved.as_deref(), Some("http://127.0.0.1:3002"));
}

#[test]
fn resolve_runtime_embedding_base_url_prefers_litellm_api_base_for_litellm_backend() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.client_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());

    let resolved =
        resolve_runtime_embedding_base_url(&settings, RuntimeEmbeddingBackendMode::LiteLlmRs);
    assert_eq!(resolved.as_deref(), Some("http://127.0.0.1:11434"));
}

#[test]
fn resolve_runtime_embedding_base_url_prefers_litellm_api_base_for_openai_backend() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.client_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:1234".to_string());

    let resolved =
        resolve_runtime_embedding_base_url(&settings, RuntimeEmbeddingBackendMode::OpenAiHttp);
    assert_eq!(resolved.as_deref(), Some("http://127.0.0.1:1234"));
}

#[test]
fn resolve_runtime_embedding_base_url_uses_none_for_mistral_sdk_backend() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.client_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:1234".to_string());
    settings.mistral.base_url = Some("http://127.0.0.1:11500".to_string());

    let resolved =
        resolve_runtime_embedding_base_url(&settings, RuntimeEmbeddingBackendMode::MistralSdk);
    assert_eq!(resolved, None);
}

#[test]
fn resolve_runtime_embedding_backend_mode_prefers_memory_override() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_backend = Some("http".to_string());
    settings.embedding.backend = Some("litellm_rs".to_string());

    let resolved = resolve_runtime_embedding_backend_mode(&settings);
    assert_eq!(resolved, RuntimeEmbeddingBackendMode::Http);
}

#[test]
fn resolve_runtime_embedding_base_url_prefers_memory_base_url_override() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.client_url = Some("http://127.0.0.1:3900".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());

    let resolved = resolve_runtime_embedding_base_url(&settings, RuntimeEmbeddingBackendMode::Http);
    assert_eq!(resolved.as_deref(), Some("http://127.0.0.1:3002"));
}

#[test]
fn resolve_runtime_embedding_base_url_prefers_litellm_api_base_over_memory_base_for_litellm() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.embedding.client_url = Some("http://127.0.0.1:3900".to_string());

    let resolved =
        resolve_runtime_embedding_base_url(&settings, RuntimeEmbeddingBackendMode::LiteLlmRs);
    assert_eq!(resolved.as_deref(), Some("http://127.0.0.1:11434"));
}

#[test]
fn resolve_runtime_memory_options_uses_embedding_timeout_secs_as_memory_default() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.timeout_secs = Some(42);

    let resolved = resolve_runtime_memory_options(&settings);
    assert_eq!(resolved.config.embedding_timeout_ms, Some(42_000));
    let _ = resolved.embedding_backend_mode;
}

#[test]
fn resolve_runtime_memory_options_prefers_memory_timeout_over_embedding_timeout_secs() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.timeout_secs = Some(90);
    settings.memory.embedding_timeout_ms = Some(5_500);

    let resolved = resolve_runtime_memory_options(&settings);
    assert_eq!(resolved.config.embedding_timeout_ms, Some(5_500));
    let _ = resolved.embedding_backend_mode;
}
