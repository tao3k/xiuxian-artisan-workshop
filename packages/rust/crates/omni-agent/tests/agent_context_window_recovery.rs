//! Agent context-window recovery tests for overflow and retried completion.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use omni_agent::{Agent, AgentConfig, set_config_home_override};

#[derive(Clone)]
struct LlmRecoveryState {
    calls: Arc<AtomicUsize>,
}

async fn handle_context_overflow_then_success(
    State(state): State<LlmRecoveryState>,
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let call_index = state.calls.fetch_add(1, Ordering::SeqCst);
    if call_index == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "message": "litellm.APIConnectionError: MinimaxException - {\"type\":\"error\",\"error\":{\"type\":\"bad_request_error\",\"message\":\"invalid params, context window exceeds limit (2013)\",\"http_code\":\"400\"},\"request_id\":\"req-1\"}",
                    "type": "server_error"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "context repaired",
                    "tool_calls": null
                },
                "finish_reason": "stop"
            }]
        })),
    )
}

async fn handle_non_context_error(
    State(state): State<LlmRecoveryState>,
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let _ = state.calls.fetch_add(1, Ordering::SeqCst);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": {
                "message": "provider temporary failure",
                "type": "server_error"
            }
        })),
    )
}

async fn spawn_context_overflow_then_success_server()
-> Result<Option<(String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>)>> {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = LlmRecoveryState {
        calls: Arc::clone(&calls),
    };
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(handle_context_overflow_then_success),
        )
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping context window recovery tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some((
        format!("http://{addr}/v1/chat/completions"),
        calls,
        handle,
    )))
}

async fn spawn_non_context_error_server()
-> Result<Option<(String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>)>> {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = LlmRecoveryState {
        calls: Arc::clone(&calls),
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(handle_non_context_error))
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping context window recovery tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some((
        format!("http://{addr}/v1/chat/completions"),
        calls,
        handle,
    )))
}

fn ensure_http_llm_backend_for_tests() {
    static CONFIG_HOME: OnceLock<PathBuf> = OnceLock::new();
    let path = CONFIG_HOME.get_or_init(|| {
        let root = std::env::temp_dir()
            .join("omni-agent-tests")
            .join("agent_context_window_recovery");
        let settings_dir = root.join("xiuxian-artisan-workshop");
        if let Err(error) = std::fs::create_dir_all(&settings_dir) {
            panic!("create isolated config home for tests: {error}");
        }
        if let Err(error) = std::fs::write(
            settings_dir.join("xiuxian.toml"),
            "[agent]\nllm_backend = \"http\"\nagenda_validation_policy = \"never\"\n",
        ) {
            panic!("write isolated runtime settings for tests: {error}");
        }
        root
    });
    set_config_home_override(path.clone());
}

fn base_agent_config(inference_url: String) -> AgentConfig {
    ensure_http_llm_backend_for_tests();
    AgentConfig {
        inference_url,
        model: "test-model".to_string(),
        api_key: None,
        max_tool_rounds: 1,
        context_budget_tokens: Some(4096),
        context_budget_reserve_tokens: 256,
        ..AgentConfig::default()
    }
}

#[tokio::test]
async fn run_turn_auto_recovers_from_context_window_error() -> Result<()> {
    let Some((inference_url, calls, server)) = spawn_context_overflow_then_success_server().await?
    else {
        return Ok(());
    };

    let agent = Agent::from_config(base_agent_config(inference_url)).await?;
    let out = agent.run_turn("ctx-repair", "hello").await?;
    assert_eq!(out, "context repaired");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "agent should retry once after context-window overflow"
    );

    server.abort();
    let _ = server.await;
    Ok(())
}

#[tokio::test]
async fn run_turn_does_not_retry_for_non_context_error() -> Result<()> {
    let Some((inference_url, calls, server)) = spawn_non_context_error_server().await? else {
        return Ok(());
    };

    let agent = Agent::from_config(base_agent_config(inference_url)).await?;
    let result = agent.run_turn("ctx-no-repair", "hello").await;
    assert!(result.is_err());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "non-context server error should not trigger retry loop"
    );

    server.abort();
    let _ = server.await;
    Ok(())
}
