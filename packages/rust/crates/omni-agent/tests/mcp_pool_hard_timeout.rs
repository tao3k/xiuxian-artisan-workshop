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

//! MCP pool hard-timeout smoke tests for omni-agent MCP facade.
//!
//! Detailed timeout and timeout-budget behavior lives in
//! `xiuxian-llm/tests/mcp_pool_hard_timeout.rs`.

use std::future::pending;
use std::time::{Duration, Instant};

use axum::Router;
use omni_agent::{McpPoolConnectConfig, connect_pool};
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorData, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

#[derive(Clone, Default)]
struct HangingMcpServer;

impl ServerHandler for HangingMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move { pending::<Result<ListToolsResult, ErrorData>>().await }
    }

    fn call_tool(
        &self,
        _request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        async move { pending::<Result<CallToolResult, ErrorData>>().await }
    }
}

async fn spawn_hanging_server(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    let service: StreamableHttpService<HangingMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(HangingMcpServer),
            std::sync::Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
                ..Default::default()
            },
        );
    let router = Router::new().nest_service("/sse", service);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind hanging mcp listener");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    })
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve local addr");
    let addr = probe.local_addr().expect("read reserved local addr");
    drop(probe);
    addr
}

fn hard_timeout_test_config() -> McpPoolConnectConfig {
    McpPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 1,
        connect_retries: 1,
        connect_retry_backoff_ms: 10,
        tool_timeout_secs: 1,
        list_tools_cache_ttl_ms: 1,
    }
}

#[tokio::test]
async fn mcp_pool_list_tools_hard_timeout_returns_promptly() {
    let addr = reserve_local_addr().await;
    let server = spawn_hanging_server(addr).await;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, hard_timeout_test_config())
        .await
        .expect("connect pool");

    let started = Instant::now();
    let error = pool
        .list_tools(None)
        .await
        .expect_err("list_tools should timeout");
    let elapsed = started.elapsed();
    let message = format!("{error:#}");

    assert!(
        message.contains("timed out after 1s"),
        "unexpected error message: {message}"
    );
    assert!(
        elapsed < Duration::from_secs(8),
        "hard timeout should return promptly, elapsed={elapsed:?}"
    );

    server.abort();
    let _ = server.await;
}
