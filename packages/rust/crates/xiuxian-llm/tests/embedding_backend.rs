//! Embedding backend parsing tests.

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
fn parse_backend_kind_supports_openai_and_mistral_sdk_aliases() {
    assert_eq!(
        parse_embedding_backend_kind(Some("openai_http")),
        Some(EmbeddingBackendKind::OpenAiHttp)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("mistral_sdk")),
        Some(EmbeddingBackendKind::MistralSdk)
    );
    assert_eq!(
        parse_embedding_backend_kind(Some("mistral-inproc")),
        Some(EmbeddingBackendKind::MistralSdk)
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
