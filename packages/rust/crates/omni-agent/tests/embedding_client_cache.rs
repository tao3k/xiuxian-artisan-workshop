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

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Json, Router, extract::State, routing::post};
use omni_agent::EmbeddingClient;

#[derive(Clone)]
struct EmbedState {
    calls: Arc<AtomicUsize>,
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve local addr");
    let addr = probe.local_addr().expect("read reserved local addr");
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
        .map(|items| items.len())
        .unwrap_or(0);
    let model_bias = payload
        .get("model")
        .and_then(|value| value.as_str())
        .map(|value| value.len() as f32)
        .unwrap_or(0.0);
    let vectors = (0..text_count)
        .map(|index| vec![index as f32 + model_bias, 1.0 + model_bias])
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
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind embed listener");
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

    let first = client
        .embed_batch_with_model(&texts, None)
        .await
        .expect("first embedding call should succeed");
    let second = client
        .embed_batch_with_model(&texts, None)
        .await
        .expect("second embedding call should use cache");

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

    let model_a = client
        .embed_batch_with_model(&texts, Some("m-a"))
        .await
        .expect("model-a embedding should succeed");
    let model_b = client
        .embed_batch_with_model(&texts, Some("model-long"))
        .await
        .expect("model-b embedding should succeed");
    let model_a_cached = client
        .embed_batch_with_model(&texts, Some("m-a"))
        .await
        .expect("model-a cached embedding should succeed");

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
