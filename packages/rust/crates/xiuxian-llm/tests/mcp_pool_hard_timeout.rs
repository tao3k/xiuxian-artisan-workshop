//! MCP pool hard-timeout behavior tests.
//!
//! These tests validate that a hanging MCP request is force-aborted by the pool
//! timeout path and returns promptly instead of waiting indefinitely.

use std::future::pending;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use axum::Router;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorData, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use xiuxian_llm::mcp::{McpPoolConnectConfig, connect_pool};

#[derive(Clone, Default)]
struct HangingMcpServer;

impl ServerHandler for HangingMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        pending::<Result<ListToolsResult, ErrorData>>().await
    }

    async fn call_tool(
        &self,
        _request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        pending::<Result<CallToolResult, ErrorData>>().await
    }
}

async fn spawn_hanging_server(addr: std::net::SocketAddr) -> Result<tokio::task::JoinHandle<()>> {
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
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    }))
}

async fn reserve_local_addr() -> Result<std::net::SocketAddr> {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = probe.local_addr()?;
    drop(probe);
    Ok(addr)
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

fn long_tool_timeout_test_config() -> McpPoolConnectConfig {
    McpPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 1,
        connect_retries: 1,
        connect_retry_backoff_ms: 10,
        tool_timeout_secs: 30,
        list_tools_cache_ttl_ms: 1,
    }
}

#[tokio::test]
async fn mcp_pool_list_tools_hard_timeout_returns_promptly() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let server = spawn_hanging_server(addr).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, hard_timeout_test_config(), None).await?;

    let started = Instant::now();
    let Err(error) = pool.list_tools(None).await else {
        return Err(anyhow!("list_tools should timeout"));
    };
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
    if let Err(error) = server.await
        && !error.is_cancelled()
    {
        return Err(anyhow!("server task join failed: {error}"));
    }
    Ok(())
}

#[tokio::test]
async fn mcp_pool_call_tool_hard_timeout_returns_promptly() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let server = spawn_hanging_server(addr).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, hard_timeout_test_config(), None).await?;

    let started = Instant::now();
    let result = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "hello" })),
        )
        .await;
    let Err(error) = result else {
        return Err(anyhow!("call_tool should timeout"));
    };
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
    if let Err(error) = server.await
        && !error.is_cancelled()
    {
        return Err(anyhow!("server task join failed: {error}"));
    }
    Ok(())
}

#[tokio::test]
async fn mcp_pool_memory_save_tool_uses_shorter_timeout_budget() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let server = spawn_hanging_server(addr).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, long_tool_timeout_test_config(), None).await?;

    let started = Instant::now();
    let result = pool
        .call_tool(
            "memory.save_memory".to_string(),
            Some(serde_json::json!({ "content": "hello" })),
        )
        .await;
    let Err(error) = result else {
        return Err(anyhow!("memory.save_memory should timeout"));
    };
    let elapsed = started.elapsed();
    let message = format!("{error:#}");

    assert!(
        message.contains("timed out after 5s"),
        "unexpected error message: {message}"
    );
    assert!(
        elapsed < Duration::from_secs(12),
        "memory.save_memory should not wait full global timeout, elapsed={elapsed:?}"
    );

    server.abort();
    if let Err(error) = server.await
        && !error.is_cancelled()
    {
        return Err(anyhow!("server task join failed: {error}"));
    }
    Ok(())
}
