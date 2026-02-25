#![allow(
    missing_docs,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::manual_async_fn,
    clippy::async_yields_async,
    clippy::no_effect_underscore_binding
)]

use xiuxian_llm::llm::backend::{LlmBackendKind, parse_llm_backend_kind};

#[test]
fn parse_llm_backend_kind_supports_http_aliases() {
    assert_eq!(
        parse_llm_backend_kind(Some("http")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("openai-compatible")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("openai_http")),
        Some(LlmBackendKind::OpenAiCompatibleHttp)
    );
}

#[test]
fn parse_llm_backend_kind_supports_mistral_aliases() {
    assert_eq!(
        parse_llm_backend_kind(Some("mistral_server")),
        Some(LlmBackendKind::MistralLocal)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("mistral_local")),
        Some(LlmBackendKind::MistralLocal)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("mistral-rs")),
        Some(LlmBackendKind::MistralLocal)
    );
}

#[test]
fn parse_llm_backend_kind_supports_litellm_aliases() {
    assert_eq!(
        parse_llm_backend_kind(Some("litellm_rs")),
        Some(LlmBackendKind::LiteLlmRs)
    );
    assert_eq!(
        parse_llm_backend_kind(Some("litellm-rs")),
        Some(LlmBackendKind::LiteLlmRs)
    );
}

#[test]
fn parse_llm_backend_kind_rejects_unknown_or_empty() {
    assert_eq!(parse_llm_backend_kind(Some("unsupported")), None);
    assert_eq!(parse_llm_backend_kind(Some("")), None);
    assert_eq!(parse_llm_backend_kind(None), None);
}
