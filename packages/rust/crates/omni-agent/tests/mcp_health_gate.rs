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

//! MCP health readiness gate behavior.

use std::time::{Duration, Instant};

use axum::routing::get;
use axum::{Json, Router};
use omni_agent::{Agent, AgentConfig, McpServerEntry};
use serde_json::json;

fn config_for(base_url: &str, retries: u32) -> AgentConfig {
    AgentConfig {
        mcp_servers: vec![McpServerEntry {
            name: "mock-mcp".to_string(),
            url: Some(format!("{base_url}/sse")),
            command: None,
            args: None,
        }],
        mcp_pool_size: 1,
        mcp_handshake_timeout_secs: 1,
        mcp_connect_retries: retries,
        mcp_strict_startup: true,
        mcp_connect_retry_backoff_ms: 10,
        ..Default::default()
    }
}

async fn spawn_server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("read test listener addr");
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), handle)
}

#[tokio::test]
async fn startup_fails_when_structured_health_never_becomes_ready() {
    let app = Router::new().route(
        "/health",
        get(|| async {
            Json(json!({
                "status": "ok",
                "ready": false,
                "initializing": true,
                "active_sessions": 0,
            }))
        }),
    );
    let (base_url, server_task) = spawn_server(app).await;

    let started = Instant::now();
    let error = match Agent::from_config(config_for(&base_url, 2)).await {
        Ok(_) => panic!("startup should fail when MCP health is never ready"),
        Err(error) => error,
    };
    server_task.abort();

    let message = format!("{error:#}");
    assert!(
        message.contains("MCP health ready wait timed out"),
        "unexpected error message: {message}"
    );
    assert!(
        started.elapsed() >= Duration::from_millis(900),
        "health readiness gate should wait before failing"
    );
}

#[tokio::test]
async fn startup_keeps_legacy_handshake_path_when_health_is_not_structured() {
    let app = Router::new().route("/health", get(|| async { "ok" }));
    let (base_url, server_task) = spawn_server(app).await;

    let error = match Agent::from_config(config_for(&base_url, 1)).await {
        Ok(_) => panic!("startup should fail because /sse is not an MCP endpoint"),
        Err(error) => error,
    };
    server_task.abort();

    let message = format!("{error:#}");
    assert!(
        !message.contains("MCP health ready wait timed out"),
        "health gate should be skipped for non-structured health endpoints: {message}"
    );
    assert!(
        message.contains("MCP connect failed after 1 attempts"),
        "unexpected error message: {message}"
    );
}
