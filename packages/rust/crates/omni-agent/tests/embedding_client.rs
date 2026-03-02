//! Integration tests for embedding client transport selection and fallback.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use omni_agent::EmbeddingClient;
use serde_json::json;

#[derive(Clone)]
struct EmbedTestState {
    http_delay: Duration,
    http_fail: bool,
    http_fail_first: bool,
    openai_fail: bool,
    http_calls: Arc<AtomicUsize>,
    mcp_calls: Arc<AtomicUsize>,
    litellm_calls: Arc<AtomicUsize>,
}

fn http_vector_score(text: &str) -> f32 {
    let score_mod = text
        .as_bytes()
        .iter()
        .fold(0_u32, |acc, byte| acc.saturating_add(u32::from(*byte)))
        % 10_000;
    let score_mod = u16::try_from(score_mod).unwrap_or(u16::MAX);
    f32::from(score_mod) / 1000.0
}

fn http_vectors_for_texts(texts: &[String]) -> Vec<Vec<f32>> {
    texts
        .iter()
        .map(|text| vec![http_vector_score(text), 1.0_f32])
        .collect()
}

fn openai_vectors_for_texts(texts: &[String]) -> Vec<Vec<f32>> {
    texts
        .iter()
        .map(|text| vec![http_vector_score(text), 7.0_f32])
        .collect()
}

async fn handle_embed_batch(
    State(state): State<EmbedTestState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let call_index = state.http_calls.fetch_add(1, Ordering::Relaxed) + 1;
    tokio::time::sleep(state.http_delay).await;
    if state.http_fail || (state.http_fail_first && call_index == 1) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "embed backend unavailable"
            })),
        );
    }
    let texts = payload
        .get("texts")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    (
        StatusCode::OK,
        Json(json!({
            "vectors": http_vectors_for_texts(&texts)
        })),
    )
}

async fn handle_mcp_embed(State(state): State<EmbedTestState>) -> Json<serde_json::Value> {
    state.mcp_calls.fetch_add(1, Ordering::Relaxed);
    Json(json!({
        "jsonrpc": "2.0",
        "id": "mcp-embed",
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "{\"success\":true,\"vectors\":[[2.0,2.0]]}"
                }
            ]
        }
    }))
}

async fn handle_litellm_embeddings(
    State(state): State<EmbedTestState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.litellm_calls.fetch_add(1, Ordering::Relaxed);
    if state.openai_fail {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "openai-compatible embedding unavailable"
            })),
        );
    }
    let texts = payload
        .get("input")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let vectors = openai_vectors_for_texts(&texts);
    let data: Vec<serde_json::Value> = vectors
        .into_iter()
        .enumerate()
        .map(|(index, embedding)| {
            json!({
                "object": "embedding",
                "index": index,
                "embedding": embedding
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(json!({
            "object": "list",
            "data": data,
            "model": "test-embed-model",
            "usage": {"prompt_tokens": 0, "total_tokens": 0}
        })),
    )
}

type SpawnedEmbeddingServer = (String, Arc<AtomicUsize>, Arc<AtomicUsize>, Arc<AtomicUsize>);

fn require_vectors(vectors: Option<Vec<Vec<f32>>>, context: &str) -> Vec<Vec<f32>> {
    match vectors {
        Some(vectors) => vectors,
        None => panic!("{context}"),
    }
}

async fn spawn_embedding_mock_server(
    http_delay: Duration,
    http_fail: bool,
    http_fail_first: bool,
) -> Result<Option<SpawnedEmbeddingServer>> {
    spawn_embedding_mock_server_with_openai_failure(http_delay, http_fail, http_fail_first, false)
        .await
}

async fn spawn_embedding_mock_server_with_openai_failure(
    http_delay: Duration,
    http_fail: bool,
    http_fail_first: bool,
    openai_fail: bool,
) -> Result<Option<SpawnedEmbeddingServer>> {
    let http_calls = Arc::new(AtomicUsize::new(0));
    let mcp_calls = Arc::new(AtomicUsize::new(0));
    let litellm_calls = Arc::new(AtomicUsize::new(0));
    let state = EmbedTestState {
        http_delay,
        http_fail,
        http_fail_first,
        openai_fail,
        http_calls: Arc::clone(&http_calls),
        mcp_calls: Arc::clone(&mcp_calls),
        litellm_calls: Arc::clone(&litellm_calls),
    };
    let app = Router::new()
        .route("/embed/batch", post(handle_embed_batch))
        .route("/messages/", post(handle_mcp_embed))
        .route("/v1/embeddings", post(handle_litellm_embeddings))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping embedding client tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some((
        format!("http://{addr}"),
        http_calls,
        mcp_calls,
        litellm_calls,
    )))
}

#[tokio::test]
async fn embed_batch_prefers_http_primary_even_when_mcp_is_faster() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(900), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &base_url,
        5,
        Some(format!("{base_url}/messages/")),
        Some("http"),
    );
    let texts = vec!["hello".to_string()];
    let started = std::time::Instant::now();
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "expected embeddings from primary HTTP path",
    );
    let elapsed = started.elapsed();

    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert!(
        elapsed >= Duration::from_millis(700),
        "expected HTTP-first completion, got elapsed={elapsed:?}"
    );
    assert_eq!(http_calls.load(Ordering::Relaxed), 1);
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[tokio::test]
async fn embed_batch_returns_none_when_http_fails_even_if_mcp_url_is_set() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), true, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &base_url,
        5,
        Some(format!("{base_url}/messages/")),
        Some("http"),
    );
    let texts = vec!["hello".to_string()];
    let vectors = client.embed_batch_with_model(&texts, None).await;
    assert!(vectors.is_none());
    assert!(
        http_calls.load(Ordering::Relaxed) >= 2,
        "persistent server error should trigger at least one retry on HTTP path"
    );
    assert_eq!(
        mcp_calls.load(Ordering::Relaxed),
        0,
        "rust-only mode disables MCP fallback even when MCP URL is configured"
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_retries_once_on_transient_http_server_error() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, true).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &base_url,
        5,
        Some(format!("{base_url}/messages/")),
        Some("http"),
    );
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "expected embeddings from retried HTTP path",
    );

    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        2,
        "transient server error should be recovered by one retry"
    );
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[tokio::test]
async fn embed_batch_falls_back_to_http_when_mcp_unconfigured() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_mcp_url_and_backend(&base_url, 5, None, Some("http"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "expected embeddings from http fallback path",
    );
    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(http_calls.load(Ordering::Relaxed), 1);
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[tokio::test]
async fn embed_batch_litellm_ollama_prefers_openai_http_direct_path() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client =
        EmbeddingClient::new_with_mcp_url_and_backend(&base_url, 5, None, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("ollama/qwen3-embedding:0.6b"))
            .await,
        "expected embeddings from OpenAI-compatible direct path",
    );
    assert_eq!(vectors, openai_vectors_for_texts(&texts));
    assert_eq!(http_calls.load(Ordering::Relaxed), 0);
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        litellm_calls.load(Ordering::Relaxed),
        1,
        "ollama direct path should call /v1/embeddings once",
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_openai_backend_uses_v1_embeddings_endpoint() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client =
        EmbeddingClient::new_with_mcp_url_and_backend(&base_url, 5, None, Some("openai_http"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("qwen3-embedding:0.6b"))
            .await,
        "expected embeddings from /v1/embeddings",
    );
    assert_eq!(vectors, openai_vectors_for_texts(&texts));
    assert_eq!(http_calls.load(Ordering::Relaxed), 0);
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    assert_eq!(litellm_calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_mistral_falls_back_to_http_without_mcp_when_provider_fails()
-> Result<()> {
    let Some((base_url, http_calls, mcp_calls, litellm_calls)) =
        spawn_embedding_mock_server_with_openai_failure(
            Duration::from_millis(5),
            false,
            false,
            true,
        )
        .await?
    else {
        return Ok(());
    };

    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &base_url,
        5,
        Some(format!("{base_url}/messages/")),
        Some("litellm_rs"),
    );
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("mistral/mistral-embed"))
            .await,
        "expected embeddings from /embed/batch fallback",
    );

    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        1,
        "expected one /embed/batch fallback request"
    );
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    assert!(
        litellm_calls.load(Ordering::Relaxed) <= 1,
        "provider path should be attempted at most once before http fallback"
    );
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_mistral_returns_none_when_provider_and_http_fail() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, litellm_calls)) =
        spawn_embedding_mock_server_with_openai_failure(
            Duration::from_millis(5),
            true,
            false,
            true,
        )
        .await?
    else {
        return Ok(());
    };

    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &base_url,
        5,
        Some(format!("{base_url}/messages/")),
        Some("litellm_rs"),
    );
    let texts = vec!["hello".to_string()];
    let vectors = client
        .embed_batch_with_model(&texts, Some("mistral/mistral-embed"))
        .await;

    assert!(vectors.is_none());
    assert!(
        http_calls.load(Ordering::Relaxed) >= 1,
        "expected /embed/batch fallback attempts when provider path fails"
    );
    assert!(
        litellm_calls.load(Ordering::Relaxed) <= 1,
        "provider path should be attempted at most once before fallback chain completes"
    );
    assert_eq!(
        mcp_calls.load(Ordering::Relaxed),
        0,
        "rust-only mode disables MCP fallback for mistral provider failures"
    );
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_ollama_direct_path_ignores_embed_batch_errors() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), true, false).await?
    else {
        return Ok(());
    };
    let client =
        EmbeddingClient::new_with_mcp_url_and_backend(&base_url, 5, None, Some("litellm_rs"));
    let texts = vec!["hello".to_string()];
    let vectors = require_vectors(
        client
            .embed_batch_with_model(&texts, Some("ollama/qwen3-embedding:0.6b"))
            .await,
        "expected embeddings from OpenAI-compatible fallback path",
    );

    assert_eq!(vectors, openai_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        0,
        "ollama direct path should skip /embed/batch when OpenAI-compatible endpoint is available"
    );
    assert_eq!(mcp_calls.load(Ordering::Relaxed), 0);
    assert_eq!(litellm_calls.load(Ordering::Relaxed), 1);
    Ok(())
}

#[cfg(feature = "agent-provider-litellm")]
#[tokio::test]
async fn embed_batch_litellm_ollama_returns_none_when_all_primary_paths_fail() -> Result<()> {
    let Some((base_url, http_calls, mcp_calls, litellm_calls)) =
        spawn_embedding_mock_server_with_openai_failure(
            Duration::from_millis(5),
            true,
            false,
            true,
        )
        .await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_mcp_url_and_backend(
        &base_url,
        5,
        Some(format!("{base_url}/messages/")),
        Some("litellm_rs"),
    );
    let texts = vec!["hello".to_string()];
    let vectors = client
        .embed_batch_with_model(&texts, Some("ollama/qwen3-embedding:0.6b"))
        .await;

    assert!(vectors.is_none());
    assert!(
        http_calls.load(Ordering::Relaxed) >= 1,
        "expected /embed/batch fallback attempts before marking embedding unavailable"
    );
    assert!(
        litellm_calls.load(Ordering::Relaxed) >= 1,
        "expected OpenAI-compatible path to be attempted before failure"
    );
    assert_eq!(
        mcp_calls.load(Ordering::Relaxed),
        0,
        "rust-only mode disables MCP fallback when all primary embedding paths fail"
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_splits_payload_by_chunk_size_and_preserves_order() -> Result<()> {
    let Some((base_url, http_calls, _mcp_calls, _litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(5), false, false).await?
    else {
        return Ok(());
    };
    let client = EmbeddingClient::new_with_mcp_url_and_backend_and_tuning(
        &base_url,
        5,
        None,
        Some("http"),
        Some(2),
        Some(1),
    );
    let texts = vec![
        "chunk-0".to_string(),
        "chunk-1".to_string(),
        "chunk-2".to_string(),
        "chunk-3".to_string(),
        "chunk-4".to_string(),
    ];
    let vectors = require_vectors(
        client.embed_batch_with_model(&texts, None).await,
        "chunked embedding should succeed",
    );
    assert_eq!(vectors, http_vectors_for_texts(&texts));
    assert_eq!(
        http_calls.load(Ordering::Relaxed),
        3,
        "5 texts with chunk_size=2 should trigger 3 HTTP calls"
    );
    Ok(())
}

#[tokio::test]
async fn embed_batch_chunk_concurrency_reduces_wall_time() -> Result<()> {
    let texts = vec![
        "alpha".to_string(),
        "bravo".to_string(),
        "charlie".to_string(),
        "delta".to_string(),
        "echo".to_string(),
        "foxtrot".to_string(),
    ];

    let Some((seq_url, seq_http_calls, _seq_mcp_calls, _seq_litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(200), false, false).await?
    else {
        return Ok(());
    };
    let seq_client = EmbeddingClient::new_with_mcp_url_and_backend_and_tuning(
        &seq_url,
        5,
        None,
        Some("http"),
        Some(2),
        Some(1),
    );
    let seq_started = std::time::Instant::now();
    let seq_vectors = require_vectors(
        seq_client.embed_batch_with_model(&texts, None).await,
        "sequential chunked embedding should succeed",
    );
    let seq_elapsed = seq_started.elapsed();
    assert_eq!(seq_vectors, http_vectors_for_texts(&texts));
    assert_eq!(seq_http_calls.load(Ordering::Relaxed), 3);

    let Some((con_url, con_http_calls, _con_mcp_calls, _con_litellm_calls)) =
        spawn_embedding_mock_server(Duration::from_millis(200), false, false).await?
    else {
        return Ok(());
    };
    let con_client = EmbeddingClient::new_with_mcp_url_and_backend_and_tuning(
        &con_url,
        5,
        None,
        Some("http"),
        Some(2),
        Some(3),
    );
    let con_started = std::time::Instant::now();
    let con_vectors = require_vectors(
        con_client.embed_batch_with_model(&texts, None).await,
        "concurrent chunked embedding should succeed",
    );
    let con_elapsed = con_started.elapsed();
    assert_eq!(con_vectors, http_vectors_for_texts(&texts));
    assert_eq!(con_http_calls.load(Ordering::Relaxed), 3);

    assert!(
        con_elapsed + Duration::from_millis(180) < seq_elapsed,
        "expected concurrent chunk execution to reduce wall time (seq={seq_elapsed:?}, concurrent={con_elapsed:?})"
    );
    Ok(())
}
