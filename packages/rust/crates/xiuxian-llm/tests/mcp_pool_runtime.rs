//! MCP pool runtime integration tests.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use axum::Router;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ErrorData, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use xiuxian_llm::mcp::{McpPoolConnectConfig, connect_pool};

#[derive(Clone)]
struct MockMcpServer {
    list_tools_calls: Arc<AtomicUsize>,
}

impl MockMcpServer {
    fn new(list_tools_calls: Arc<AtomicUsize>) -> Self {
        Self { list_tools_calls }
    }

    fn mock_tool() -> Tool {
        Tool {
            name: "test.ping".into(),
            title: Some("Ping".into()),
            description: Some("mock tool".into()),
            input_schema: Arc::new(serde_json::Map::new()),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        }
    }
}

impl ServerHandler for MockMcpServer {
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
        self.list_tools_calls.fetch_add(1, Ordering::SeqCst);
        std::future::ready(Ok(ListToolsResult::with_all_items(vec![Self::mock_tool()])))
    }

    fn call_tool(
        &self,
        _request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(CallToolResult::success(vec![])))
    }
}

async fn reserve_local_addr() -> Result<std::net::SocketAddr> {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = probe.local_addr()?;
    drop(probe);
    Ok(addr)
}

async fn spawn_mock_server(
    addr: std::net::SocketAddr,
) -> Result<(tokio::task::JoinHandle<()>, Arc<AtomicUsize>)> {
    let list_tools_calls = Arc::new(AtomicUsize::new(0));
    let service: StreamableHttpService<MockMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let list_tools_calls = Arc::clone(&list_tools_calls);
                move || Ok(MockMcpServer::new(Arc::clone(&list_tools_calls)))
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
                ..Default::default()
            },
        );
    let router = Router::new().nest_service("/sse", service);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok((
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        }),
        list_tools_calls,
    ))
}

fn test_connect_config() -> McpPoolConnectConfig {
    McpPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 2,
        connect_retries: 3,
        connect_retry_backoff_ms: 100,
        tool_timeout_secs: 10,
        list_tools_cache_ttl_ms: 1_000,
    }
}

#[tokio::test]
async fn mcp_pool_list_tools_cache_serves_second_request_from_cache() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let (handle, list_tools_calls) = spawn_mock_server(addr).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, test_connect_config(), None).await?;

    let first = pool.list_tools(None).await?;
    assert_eq!(first.tools.len(), 1);

    let second = pool.list_tools(None).await?;
    assert_eq!(second.tools.len(), 1);

    assert_eq!(
        list_tools_calls.load(Ordering::SeqCst),
        1,
        "second list_tools should be served from cache"
    );
    let stats = pool.tools_list_cache_stats_snapshot();
    assert_eq!(stats.requests_total, 2);
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.cache_misses, 1);
    assert_eq!(stats.cache_refreshes, 1);

    handle.abort();
    if let Err(error) = handle.await
        && !error.is_cancelled()
    {
        return Err(anyhow::anyhow!("mock server task join failed: {error}"));
    }
    Ok(())
}

#[tokio::test]
async fn mcp_pool_discover_cache_stats_absent_when_not_configured() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let (handle, _) = spawn_mock_server(addr).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, test_connect_config(), None).await?;

    assert!(
        pool.discover_cache_stats_snapshot().is_none(),
        "discover cache stats should be absent without discover cache backend"
    );

    handle.abort();
    if let Err(error) = handle.await
        && !error.is_cancelled()
    {
        return Err(anyhow::anyhow!("mock server task join failed: {error}"));
    }
    Ok(())
}
