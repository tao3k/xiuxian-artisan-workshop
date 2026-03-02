use super::{EmbeddingBackendMode, parse_backend_mode};

#[test]
fn parse_backend_mode_supports_openai_and_mistral_sdk_aliases() {
    assert_eq!(
        parse_backend_mode(Some("openai_http")),
        EmbeddingBackendMode::OpenAiHttp
    );
    assert_eq!(
        parse_backend_mode(Some("mistral_sdk")),
        EmbeddingBackendMode::MistralSdk
    );
}

#[test]
fn parse_backend_mode_retains_legacy_http_alias() {
    assert_eq!(parse_backend_mode(Some("http")), EmbeddingBackendMode::Http);
    assert_eq!(
        parse_backend_mode(Some("client")),
        EmbeddingBackendMode::Http
    );
}
