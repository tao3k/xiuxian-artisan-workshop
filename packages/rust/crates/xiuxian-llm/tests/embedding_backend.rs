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

use xiuxian_llm::embedding::backend::{EmbeddingBackendKind, parse_embedding_backend_kind};

#[test]
fn parse_backend_kind_supports_legacy_http_aliases() {
    assert_eq!(
        parse_embedding_backend_kind(Some("http")),
        Some(EmbeddingBackendKind::Http)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("client")),
        Some(EmbeddingBackendKind::Http)
    );
}

#[test]
fn parse_backend_kind_supports_openai_and_mistral_aliases() {
    assert_eq!(
        parse_embedding_backend_kind(Some("openai_http")),
        Some(EmbeddingBackendKind::OpenAiHttp)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("mistral_rs")),
        Some(EmbeddingBackendKind::MistralLocal)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("mistral-http")),
        Some(EmbeddingBackendKind::MistralLocal)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("mistral_server")),
        Some(EmbeddingBackendKind::MistralLocal)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("mistral_local")),
        Some(EmbeddingBackendKind::MistralLocal)
    );
}

#[test]
fn parse_backend_kind_supports_litellm_aliases() {
    assert_eq!(
        parse_embedding_backend_kind(Some("litellm_rs")),
        Some(EmbeddingBackendKind::LiteLlmRs)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("provider")),
        Some(EmbeddingBackendKind::LiteLlmRs)
    );
}

#[test]
fn parse_backend_kind_rejects_unknown_or_empty() {
    assert_eq!(parse_embedding_backend_kind(Some("unknown")), None);
    assert_eq!(parse_embedding_backend_kind(Some("")), None);
    assert_eq!(parse_embedding_backend_kind(None), None);
}
