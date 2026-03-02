//! Embedding client cache behavior tests with a mock HTTP embedding endpoint.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Json, Router, extract::State, routing::post};
use omni_agent::EmbeddingClient;

#[derive(Clone)]
struct EmbedState {
    calls: Arc<AtomicUsize>,
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => panic!("reserve local addr: {error}"),
    };
    let addr = match probe.local_addr() {
        Ok(addr) => addr,
        Err(error) => panic!("read reserved local addr: {error}"),
    };
    drop(probe);
    addr
}

async fn embed_batch_handler(
    State(state): State<EmbedState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    state.calls.fetch_add(1, Ordering::Relaxed);
    let text_count = payload
        .get("texts")
        .and_then(|value| value.as_array())
        .map_or(0, Vec::len);
    let model_bias = payload
        .get("model")
        .and_then(|value| value.as_str())
        .map_or(0.0, |value| {
            let model_len = u16::try_from(value.len()).unwrap_or(u16::MAX);
            f32::from(model_len)
        });
    let vectors = (0..text_count)
        .map(|index| {
            let index_f32 = f32::from(u16::try_from(index).unwrap_or(u16::MAX));
            vec![index_f32 + model_bias, 1.0 + model_bias]
        })
        .collect::<Vec<Vec<f32>>>();
    Json(serde_json::json!({ "vectors": vectors }))
}

async fn spawn_embed_server(
    addr: std::net::SocketAddr,
    calls: Arc<AtomicUsize>,
) -> tokio::task::JoinHandle<()> {
    let app = Router::new()
        .route("/embed/batch", post(embed_batch_handler))
        .with_state(EmbedState { calls });
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => panic!("bind embed listener: {error}"),
    };
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}

#[tokio::test]
async fn repeated_embedding_batch_uses_local_cache() {
    let addr = reserve_local_addr().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let server = spawn_embed_server(addr, calls.clone()).await;
    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &format!("http://{addr}"),
        5,
        None,
        Some("http"),
    );
    let texts = vec!["repeat this prompt".to_string()];

    let first = client.embed_batch_with_model(&texts, None).await;
    let Some(first) = first else {
        panic!("first embedding call should succeed");
    };
    let second = client.embed_batch_with_model(&texts, None).await;
    let Some(second) = second else {
        panic!("second embedding call should use cache");
    };

    assert_eq!(first, second);
    assert_eq!(
        calls.load(Ordering::Relaxed),
        1,
        "second call should be served from local embedding cache"
    );

    server.abort();
}

#[tokio::test]
async fn embedding_cache_isolated_by_model_hint() {
    let addr = reserve_local_addr().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let server = spawn_embed_server(addr, calls.clone()).await;
    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &format!("http://{addr}"),
        5,
        None,
        Some("http"),
    );
    let texts = vec!["same text".to_string()];

    let model_a = client.embed_batch_with_model(&texts, Some("m-a")).await;
    let Some(model_a) = model_a else {
        panic!("model-a embedding should succeed");
    };
    let model_b = client
        .embed_batch_with_model(&texts, Some("model-long"))
        .await;
    let Some(model_b) = model_b else {
        panic!("model-b embedding should succeed");
    };
    let model_a_cached = client.embed_batch_with_model(&texts, Some("m-a")).await;
    let Some(model_a_cached) = model_a_cached else {
        panic!("model-a cached embedding should succeed");
    };

    assert_eq!(model_a, model_a_cached);
    assert_ne!(
        model_a, model_b,
        "model hint should participate in cache key isolation"
    );
    assert_eq!(
        calls.load(Ordering::Relaxed),
        2,
        "two distinct model hints should populate two cache entries"
    );

    server.abort();
}
