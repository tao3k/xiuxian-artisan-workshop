//! Integration tests for OpenAI-compatible embedding transport behavior.
//!
//! Covers valid JSON decoding and non-JSON success payload handling.

use anyhow::{Context, Result};
use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::routing::post;
use std::time::Duration;
use xiuxian_llm::embedding::openai_compat::embed_openai_compatible;

#[derive(Clone)]
struct OpenAiCompatMockState {
    status: StatusCode,
    content_type: &'static str,
    body: &'static str,
}

async fn handle_embeddings(
    State(state): State<OpenAiCompatMockState>,
) -> (
    StatusCode,
    [(header::HeaderName, &'static str); 1],
    &'static str,
) {
    (
        state.status,
        [(header::CONTENT_TYPE, state.content_type)],
        state.body,
    )
}

async fn spawn_openai_compat_server(state: OpenAiCompatMockState) -> Result<Option<String>> {
    let app = Router::new()
        .route("/v1/embeddings", post(handle_embeddings))
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return Ok(None),
        Err(err) => return Err(err).context("failed to bind OpenAI-compatible test server"),
    };
    let addr = listener
        .local_addr()
        .context("test server should expose a local address")?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some(format!("http://{addr}")))
}

async fn reserve_local_port() -> Result<Option<u16>> {
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => return Ok(None),
        Err(error) => {
            return Err(error).context("failed to reserve local port for openai-compat test");
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
async fn embed_openai_compatible_parses_valid_json_payload() -> Result<()> {
    let Some(base_url) = spawn_openai_compat_server(OpenAiCompatMockState {
        status: StatusCode::OK,
        content_type: "application/json",
        body: r#"{"data":[{"embedding":[0.1,0.2,0.3]}]}"#,
    })
    .await?
    else {
        return Ok(());
    };

    let client = reqwest::Client::new();
    let texts = vec!["hello".to_string()];

    let vectors =
        embed_openai_compatible(&client, &base_url, &texts, Some("qwen3-embedding:0.6b")).await;

    assert_eq!(vectors, Some(vec![vec![0.1_f32, 0.2_f32, 0.3_f32]]));
    Ok(())
}

#[tokio::test]
async fn embed_openai_compatible_returns_none_on_non_json_success_payload() -> Result<()> {
    let Some(base_url) = spawn_openai_compat_server(OpenAiCompatMockState {
        status: StatusCode::OK,
        content_type: "text/plain",
        body: "service unavailable",
    })
    .await?
    else {
        return Ok(());
    };

    let client = reqwest::Client::new();
    let texts = vec!["hello".to_string()];

    let vectors =
        embed_openai_compatible(&client, &base_url, &texts, Some("qwen3-embedding:0.6b")).await;

    assert!(vectors.is_none());
    Ok(())
}

#[tokio::test]
async fn embed_openai_compatible_retries_connection_refused_until_server_is_ready() -> Result<()> {
    let Some(port) = reserve_local_port().await? else {
        return Ok(());
    };

    let state = OpenAiCompatMockState {
        status: StatusCode::OK,
        content_type: "application/json",
        body: r#"{"data":[{"embedding":[0.1,0.2,0.3]}]}"#,
    };
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(350)).await;
        let Ok(listener) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await else {
            return;
        };
        let app = Router::new()
            .route("/v1/embeddings", post(handle_embeddings))
            .with_state(state);
        let _ = axum::serve(listener, app).await;
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .context("failed to build reqwest client for openai-compat retry test")?;
    let texts = vec!["hello".to_string()];
    let base_url = format!("http://127.0.0.1:{port}");

    let vectors =
        embed_openai_compatible(&client, &base_url, &texts, Some("qwen3-embedding:0.6b")).await;

    assert_eq!(vectors, Some(vec![vec![0.1_f32, 0.2_f32, 0.3_f32]]));
    Ok(())
}
