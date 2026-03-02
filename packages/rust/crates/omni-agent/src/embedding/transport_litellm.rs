use std::time::Instant;

use litellm_rs::core::embedding::{EmbeddingOptions, embed_texts_with_options};

const OLLAMA_PLACEHOLDER_API_KEY: &str = "ollama-local";

fn normalize_openai_compatible_base_url(api_base: &str) -> String {
    let trimmed = api_base.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.ends_with("/v1") {
        return trimmed.to_string();
    }
    format!("{trimmed}/v1")
}

fn normalize_litellm_embedding_target(
    model: &str,
    api_base: &str,
    api_key: Option<&str>,
) -> (String, String, Option<String>, bool) {
    let key = api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    if let Some(stripped_model) = model.strip_prefix("ollama/") {
        let normalized_model = format!("openai/{stripped_model}");
        let normalized_base = normalize_openai_compatible_base_url(api_base);
        let normalized_key = key.or_else(|| Some(OLLAMA_PLACEHOLDER_API_KEY.to_string()));
        return (normalized_model, normalized_base, normalized_key, true);
    }
    (model.to_string(), api_base.to_string(), key, false)
}

pub(crate) async fn embed_litellm(
    model: &str,
    texts: &[String],
    api_base: &str,
    timeout_secs: u64,
    api_key: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(vec![]);
    }
    let started = Instant::now();
    let (effective_model, effective_api_base, effective_api_key, ollama_compat_mode) =
        normalize_litellm_embedding_target(model, api_base, api_key);
    let mut options = EmbeddingOptions::new().with_timeout(timeout_secs);
    if !effective_api_base.trim().is_empty() {
        options = options.with_api_base(effective_api_base.clone());
    }
    if let Some(value) = effective_api_key.as_deref() {
        options = options.with_api_key(value.to_string());
    }
    if ollama_compat_mode {
        tracing::debug!(
            event = "agent.embedding.litellm.ollama_compat",
            model,
            effective_model,
            effective_api_base,
            "embedding litellm-rs using OpenAI-compatible Ollama normalization"
        );
    }
    let text_refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    match embed_texts_with_options(&effective_model, &text_refs, options).await {
        Ok(vectors) => {
            tracing::debug!(
                event = "agent.embedding.litellm.completed",
                model = effective_model,
                elapsed_ms = started.elapsed().as_millis(),
                vector_count = vectors.len(),
                "embedding litellm-rs path completed"
            );
            Some(vectors)
        }
        Err(error) => {
            tracing::warn!(
                event = "agent.embedding.litellm.failed",
                model = effective_model,
                elapsed_ms = started.elapsed().as_millis(),
                error = %error,
                "embedding litellm-rs path failed"
            );
            None
        }
    }
}
