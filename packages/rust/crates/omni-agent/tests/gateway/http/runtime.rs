/// HTTP gateway runtime helper and endpoint behavior tests.
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};
use std::net::TcpListener;

use crate::config::RuntimeSettings;
use crate::gateway::http::runtime::{
    resolve_embed_base_url, resolve_embed_model, resolve_runtime_embed_base_url,
};

#[test]
fn resolve_embed_model_prefers_configured_default_over_request_override() {
    let resolved = resolve_embed_model(
        Some("openai/qwen3-embedding:0.6b"),
        Some("ollama/qwen3-embedding:0.6b"),
    );
    let resolved = match resolved {
        Ok(model) => model,
        Err(error) => panic!("expected configured default model: {error:?}"),
    };
    assert_eq!(resolved, "ollama/qwen3-embedding:0.6b");
}

#[test]
fn resolve_embed_model_uses_requested_when_default_missing() {
    let resolved = match resolve_embed_model(Some("openai/text-embedding-3-small"), None) {
        Ok(model) => model,
        Err(error) => panic!("expected request model when no configured default exists: {error:?}"),
    };
    assert_eq!(resolved, "openai/text-embedding-3-small");
}

#[test]
fn resolve_embed_model_rejects_when_both_request_and_default_are_missing() {
    let Err(error) = resolve_embed_model(None, None) else {
        panic!("expected missing model error");
    };
    assert_eq!(error.0, StatusCode::BAD_REQUEST);
    assert!(error.1.contains("embedding model must be provided"));
}

#[test]
fn resolve_embed_base_url_prefers_litellm_api_base_for_litellm_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.embedding.client_url = Some("http://127.0.0.1:3900".to_string());

    let resolved = resolve_embed_base_url(&settings, Some("litellm_rs"));
    assert_eq!(resolved, "http://127.0.0.1:11434");
}

#[test]
fn resolve_embed_base_url_prefers_memory_base_url_for_http_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.embedding.client_url = Some("http://127.0.0.1:3900".to_string());

    let resolved = resolve_embed_base_url(&settings, Some("http"));
    assert_eq!(resolved, "http://127.0.0.1:3002");
}

#[test]
fn resolve_embed_base_url_uses_inproc_label_for_mistral_sdk_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.mistral.base_url = Some("http://127.0.0.1:11500".to_string());

    let resolved = resolve_embed_base_url(&settings, Some("mistral_sdk"));
    assert_eq!(resolved, "inproc://mistral-sdk");
}

#[test]
fn resolve_runtime_embed_base_url_ignores_mistral_base_url_for_non_mistral_backend() {
    let mut settings = RuntimeSettings::default();
    settings.memory.embedding_base_url = Some("http://127.0.0.1:3002".to_string());
    settings.embedding.litellm_api_base = Some("http://127.0.0.1:11434".to_string());
    settings.mistral.base_url = Some("http://127.0.0.1:11500".to_string());

    let resolved = resolve_runtime_embed_base_url(&settings, Some("litellm_rs"), None);
    assert_eq!(resolved, "http://127.0.0.1:11434");
}

#[test]
fn resolve_runtime_embed_base_url_uses_override_when_present() {
    let settings = RuntimeSettings::default();
    let resolved = resolve_runtime_embed_base_url(
        &settings,
        Some("openai_http"),
        Some("http://127.0.0.1:2999"),
    );
    assert_eq!(resolved, "http://127.0.0.1:2999");
}

fn reserve_local_port() -> Option<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    Some(listener.local_addr().ok()?.port())
}

async fn spawn_openai_embedding_stub(port: u16) -> Option<tokio::task::JoinHandle<()>> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .ok()?;
    let app = Router::new()
        .route(
            "/v1/models",
            get(|| async {
                Json(json!({
                    "object": "list",
                    "data": [{"id": "qwen3-embedding:0.6b", "object": "model"}]
                }))
            }),
        )
        .route(
            "/v1/embeddings",
            post(|Json(payload): Json<Value>| async move {
                let model = payload
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or("qwen3-embedding:0.6b");
                let input_count = match payload.get("input") {
                    Some(Value::String(_)) => 1usize,
                    Some(Value::Array(items)) => items.len(),
                    _ => 0usize,
                };
                let data = (0..input_count)
                    .map(|index| {
                        json!({
                            "object": "embedding",
                            "index": index,
                            "embedding": [0.11, 0.22, 0.33]
                        })
                    })
                    .collect::<Vec<_>>();
                Json(json!({
                    "object": "list",
                    "data": data,
                    "model": model,
                    "usage": {"prompt_tokens": 0, "total_tokens": 0}
                }))
            }),
        );
    Some(tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    }))
}

#[tokio::test]
async fn build_embedding_runtime_for_settings_embeds_with_openai_http_backend() {
    let Some(port) = reserve_local_port() else {
        eprintln!("skipping test: local socket bind is not permitted");
        return;
    };
    let Some(server_handle) = spawn_openai_embedding_stub(port).await else {
        eprintln!("skipping test: cannot spawn local openai embedding stub");
        return;
    };

    let mut runtime_settings = RuntimeSettings::default();
    runtime_settings.memory.embedding_backend = Some("openai_http".to_string());
    runtime_settings.memory.embedding_model = Some("qwen3-embedding:0.6b".to_string());
    runtime_settings.memory.embedding_base_url = Some(format!("http://127.0.0.1:{port}"));
    runtime_settings.embedding.timeout_secs = Some(3);

    let runtime = super::build_embedding_runtime_for_settings(&runtime_settings);

    let texts = vec!["gateway openai-http endpoint test".to_string()];
    let vectors = runtime
        .client
        .embed_batch_with_model(&texts, Some("qwen3-embedding:0.6b"))
        .await;
    let Some(vectors) = vectors else {
        panic!("expected embeddings from reachable openai endpoint");
    };

    assert_eq!(vectors.len(), 1);
    assert_eq!(vectors[0].len(), 3);
    assert!((vectors[0][0] - 0.11).abs() < 1e-6);
    assert!((vectors[0][1] - 0.22).abs() < 1e-6);
    assert!((vectors[0][2] - 0.33).abs() < 1e-6);

    server_handle.abort();
    let _ = server_handle.await;
}
