use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, routing::post};
use serde_json::json;
use std::time::Duration;

use super::embed_http;

#[derive(Clone)]
struct EmbedBatchMockState {
    vectors: Vec<Vec<f32>>,
}

async fn handle_embed_batch(State(state): State<EmbedBatchMockState>) -> Json<serde_json::Value> {
    Json(json!({ "vectors": state.vectors }))
}

async fn reserve_local_port() -> Result<Option<u16>> {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => return Ok(None),
        Err(error) => {
            return Err(error).context("failed to reserve local port for embed-http test");
        }
    };

    let port = listener
        .local_addr()
        .context("reserved listener should expose local addr")?
        .port();
    drop(listener);
    Ok(Some(port))
}

#[tokio::test]
async fn embed_http_retries_connection_refused_until_server_is_ready() -> Result<()> {
    let Some(port) = reserve_local_port().await? else {
        return Ok(());
    };

    let state = EmbedBatchMockState {
        vectors: vec![vec![0.1_f32, 0.2_f32, 0.3_f32]],
    };
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(350)).await;
        let Ok(listener) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await else {
            return;
        };
        let app = Router::new()
            .route("/embed/batch", post(handle_embed_batch))
            .with_state(state);
        let _ = axum::serve(listener, app).await;
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .context("failed to build reqwest client for embed-http test")?;
    let base_url = format!("http://127.0.0.1:{port}");
    let texts = vec!["hello".to_string()];
    let vectors = embed_http(
        &client,
        &base_url,
        &texts,
        Some("Qwen/Qwen3-Embedding-0.6B"),
    )
    .await;

    assert_eq!(vectors, Some(vec![vec![0.1_f32, 0.2_f32, 0.3_f32]]));
    Ok(())
}
