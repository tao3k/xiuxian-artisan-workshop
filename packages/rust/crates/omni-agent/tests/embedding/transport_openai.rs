//! Test coverage for omni-agent behavior.

use super::normalize_openai_embeddings_url;

#[test]
fn normalize_openai_embeddings_url_appends_v1_for_plain_host() {
    assert_eq!(
        normalize_openai_embeddings_url("http://127.0.0.1:11434"),
        Some("http://127.0.0.1:11434/v1/embeddings".to_string())
    );
}

#[test]
fn normalize_openai_embeddings_url_respects_existing_v1_suffix() {
    assert_eq!(
        normalize_openai_embeddings_url("http://127.0.0.1:18081/v1"),
        Some("http://127.0.0.1:18081/v1/embeddings".to_string())
    );
}
