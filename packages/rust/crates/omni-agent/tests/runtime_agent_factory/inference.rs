#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

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
fn resolve_runtime_inference_url_uses_mistral_base_url_for_mistral_backend() {
    let mut settings = RuntimeSettings::default();
    settings.agent.llm_backend = Some("mistral_local".to_string());
    settings.mistral.base_url = Some("http://127.0.0.1:18999".to_string());

    let resolved =
        resolve_runtime_inference_url(&settings, &[]).expect("mistral local url should resolve");
    assert_eq!(resolved, "http://127.0.0.1:18999/v1/chat/completions");
}

#[test]
fn resolve_runtime_inference_url_ignores_mistral_base_url_for_litellm_backend() {
    let mut settings = RuntimeSettings::default();
    settings.agent.llm_backend = Some("litellm_rs".to_string());
    settings.mistral.base_url = Some("http://127.0.0.1:18999".to_string());
    settings.inference.provider = Some("minimax".to_string());

    let resolved = resolve_runtime_inference_url(&settings, &[])
        .expect("litellm backend should not use mistral base url");
    assert_eq!(resolved, "https://api.minimax.io/v1/chat/completions");
}

#[test]
fn validate_inference_url_origin_rejects_same_origin_as_mcp_by_default() {
    let servers = vec![mcp_server("http://127.0.0.1:3002/sse")];
    let err =
        validate_inference_url_origin("http://127.0.0.1:3002/v1/chat/completions", &servers, false)
            .expect_err("shared origin should be rejected by default");
    let message = format!("{err:#}");
    assert!(message.contains("invalid inference URL"));
    assert!(message.contains("OMNI_AGENT_ALLOW_INFERENCE_MCP_SHARED_ORIGIN=true"));
}

#[test]
fn validate_inference_url_origin_allows_distinct_origin() {
    let servers = vec![mcp_server("http://127.0.0.1:3002/sse")];
    validate_inference_url_origin("http://127.0.0.1:4000/v1/chat/completions", &servers, false)
        .expect("distinct origin should be valid");
}

#[test]
fn validate_inference_url_origin_allows_shared_origin_when_opted_in() {
    let servers = vec![mcp_server("http://127.0.0.1:3002/sse")];
    validate_inference_url_origin("http://127.0.0.1:3002/v1/chat/completions", &servers, true)
        .expect("opt-in should allow shared origin");
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
fn parse_embedding_backend_mode_supports_mistral_aliases() {
    assert_eq!(
        parse_embedding_backend_mode(Some("mistral_rs")),
        Some(RuntimeEmbeddingBackendMode::MistralLocal)
    );
    assert_eq!(
        parse_embedding_backend_mode(Some("mistral-http")),
        Some(RuntimeEmbeddingBackendMode::MistralLocal)
    );
    assert_eq!(
        parse_embedding_backend_mode(Some("mistral_local")),
        Some(RuntimeEmbeddingBackendMode::MistralLocal)
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
}

#[test]
fn resolve_runtime_memory_options_prefers_memory_timeout_over_embedding_timeout_secs() {
    let mut settings = RuntimeSettings::default();
    settings.embedding.timeout_secs = Some(90);
    settings.memory.embedding_timeout_ms = Some(5_500);

    let resolved = resolve_runtime_memory_options(&settings);
    assert_eq!(resolved.config.embedding_timeout_ms, Some(5_500));
}
