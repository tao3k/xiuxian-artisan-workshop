//! Mistral SDK embedding configuration and fallback tests.

use std::path::PathBuf;

use xiuxian_llm::embedding::sdk::{
    embed_with_mistral_sdk, normalize_mistral_sdk_hf_cache_path, normalize_mistral_sdk_hf_revision,
    normalize_mistral_sdk_max_num_seqs, normalize_mistral_sdk_model,
};

#[test]
fn normalize_mistral_sdk_model_rejects_empty_values() {
    assert_eq!(normalize_mistral_sdk_model(None), None);
    assert_eq!(normalize_mistral_sdk_model(Some("")), None);
    assert_eq!(normalize_mistral_sdk_model(Some("   ")), None);
}

#[test]
fn normalize_mistral_sdk_model_trims_values() {
    assert_eq!(
        normalize_mistral_sdk_model(Some("  Qwen/Qwen3-Embedding-0.6B  ")),
        Some("Qwen/Qwen3-Embedding-0.6B".to_string())
    );
}

#[test]
fn normalize_mistral_sdk_hf_cache_path_rejects_empty_values() {
    assert_eq!(normalize_mistral_sdk_hf_cache_path(None), None);
    assert_eq!(normalize_mistral_sdk_hf_cache_path(Some("")), None);
    assert_eq!(normalize_mistral_sdk_hf_cache_path(Some("  ")), None);
}

#[test]
fn normalize_mistral_sdk_hf_cache_path_trims_values() {
    assert_eq!(
        normalize_mistral_sdk_hf_cache_path(Some("  .data/models/hf-cache  ")),
        Some(PathBuf::from(".data/models/hf-cache"))
    );
}

#[test]
fn normalize_mistral_sdk_hf_revision_rejects_empty_values() {
    assert_eq!(normalize_mistral_sdk_hf_revision(None), None);
    assert_eq!(normalize_mistral_sdk_hf_revision(Some("")), None);
    assert_eq!(normalize_mistral_sdk_hf_revision(Some("  ")), None);
}

#[test]
fn normalize_mistral_sdk_hf_revision_trims_values() {
    assert_eq!(
        normalize_mistral_sdk_hf_revision(Some("  main  ")),
        Some("main".to_string())
    );
}

#[test]
fn normalize_mistral_sdk_max_num_seqs_bounds_values() {
    assert_eq!(normalize_mistral_sdk_max_num_seqs(None), None);
    assert_eq!(normalize_mistral_sdk_max_num_seqs(Some(0)), None);
    assert_eq!(normalize_mistral_sdk_max_num_seqs(Some(32)), Some(32));
    assert_eq!(normalize_mistral_sdk_max_num_seqs(Some(9_999)), Some(4_096));
}

#[tokio::test]
async fn embed_with_mistral_sdk_returns_empty_for_empty_text_batch() {
    let vectors =
        embed_with_mistral_sdk(&[], Some("Qwen/Qwen3-Embedding-0.6B"), None, None, None).await;
    assert_eq!(vectors, Some(Vec::new()));
}

#[tokio::test]
async fn embed_with_mistral_sdk_rejects_missing_model() {
    let texts = vec!["hello".to_string()];
    let vectors = embed_with_mistral_sdk(&texts, None, None, None, None).await;
    assert_eq!(vectors, None);
}
