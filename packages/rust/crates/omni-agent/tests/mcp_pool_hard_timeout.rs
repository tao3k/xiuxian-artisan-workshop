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
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => panic!("bind hanging mcp listener: {error}"),
    };
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    })
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
    let pool = connect_pool(&url, hard_timeout_test_config()).await;
    let pool = match pool {
        Ok(pool) => pool,
        Err(error) => panic!("connect pool: {error}"),
    };

    let started = Instant::now();
    let Err(error) = pool.list_tools(None).await else {
        panic!("list_tools should timeout");
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
    let _ = server.await;
}
