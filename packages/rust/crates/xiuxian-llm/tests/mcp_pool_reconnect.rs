//! MCP pool reconnect integration tests.

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use xiuxian_llm::mcp::{McpPoolConnectConfig, connect_pool};

#[derive(Clone, Default)]
struct MockMcpServer {
    list_failures_remaining: Arc<AtomicUsize>,
    list_calls_total: Arc<AtomicUsize>,
}

impl MockMcpServer {
    fn mock_tool() -> Tool {
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": { "message": { "type": "string" } },
        });
        let map = input_schema.as_object().cloned().unwrap_or_default();
        Tool {
            name: "mock_echo".into(),
            title: Some("Mock Echo".into()),
            description: Some("Echo for reconnect tests".into()),
            input_schema: Arc::new(map),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        }
    }
}

#[derive(Clone)]
struct EmbeddingTimeoutMcpServer {
    call_calls_total: Arc<AtomicUsize>,
}

impl ServerHandler for EmbeddingTimeoutMcpServer {
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
        std::future::ready(Ok(ListToolsResult::with_all_items(vec![
            MockMcpServer::mock_tool(),
        ])))
    }

    fn call_tool(
        &self,
        _request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        self.call_calls_total.fetch_add(1, Ordering::SeqCst);
        std::future::ready(Err(ErrorData::internal_error(
            "Embedding timed out after 5s. Ensure MCP embedding service is running and responsive.",
            None,
        )))
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
        self.list_calls_total.fetch_add(1, Ordering::SeqCst);
        if self
            .list_failures_remaining
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                value.checked_sub(1)
            })
            .is_ok()
        {
            return std::future::ready(Err(ErrorData::internal_error(
                "simulated list_tools failure",
                None,
            )));
        }
        std::future::ready(Ok(ListToolsResult::with_all_items(vec![Self::mock_tool()])))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let msg = request
            .arguments
            .as_ref()
            .and_then(|m| m.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("ok");
        let content = CallToolResult::success(vec![Content::text(format!("echo: {msg}"))]);
        std::future::ready(Ok(content))
    }
}

async fn spawn_mock_server(
    addr: std::net::SocketAddr,
    initial_list_failures: usize,
) -> Result<tokio::task::JoinHandle<()>> {
    let (handle, _) = spawn_mock_server_with_list_counter(addr, initial_list_failures).await?;
    Ok(handle)
}

async fn spawn_mock_server_with_list_counter(
    addr: std::net::SocketAddr,
    initial_list_failures: usize,
) -> Result<(tokio::task::JoinHandle<()>, Arc<AtomicUsize>)> {
    let list_failures_remaining = Arc::new(AtomicUsize::new(initial_list_failures));
    let list_calls_total = Arc::new(AtomicUsize::new(0));
    let service: StreamableHttpService<MockMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let list_failures_remaining = list_failures_remaining.clone();
                let list_calls_total = list_calls_total.clone();
                move || {
                    Ok(MockMcpServer {
                        list_failures_remaining: list_failures_remaining.clone(),
                        list_calls_total: list_calls_total.clone(),
                    })
                }
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
                ..Default::default()
            },
        );
    let router = Router::new().nest_service("/sse", service);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind mock mcp listener on {addr}"))?;
    Ok((
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        }),
        list_calls_total,
    ))
}

async fn spawn_embedding_timeout_server(
    addr: std::net::SocketAddr,
) -> Result<(tokio::task::JoinHandle<()>, Arc<AtomicUsize>)> {
    let call_calls_total = Arc::new(AtomicUsize::new(0));
    let service: StreamableHttpService<EmbeddingTimeoutMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let call_calls_total = call_calls_total.clone();
                move || {
                    Ok(EmbeddingTimeoutMcpServer {
                        call_calls_total: call_calls_total.clone(),
                    })
                }
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
                ..Default::default()
            },
        );
    let router = Router::new().nest_service("/sse", service);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind embedding-timeout mock mcp listener on {addr}"))?;
    Ok((
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        }),
        call_calls_total,
    ))
}

fn reconnect_test_config(pool_size: usize) -> McpPoolConnectConfig {
    McpPoolConnectConfig {
        pool_size,
        handshake_timeout_secs: 1,
        connect_retries: 6,
        connect_retry_backoff_ms: 100,
        tool_timeout_secs: 10,
        list_tools_cache_ttl_ms: 1_000,
    }
}

async fn reserve_local_addr() -> Result<std::net::SocketAddr> {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("reserve local addr")?;
    let addr = probe.local_addr().context("read reserved local addr")?;
    drop(probe);
    Ok(addr)
}

#[tokio::test]
async fn mcp_pool_list_tools_cache_reuses_recent_snapshot() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let (handle, list_calls_total) = spawn_mock_server_with_list_counter(addr, 0).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, reconnect_test_config(1), None)
        .await
        .context("connect pool")?;

    let first = pool
        .list_tools(None)
        .await
        .context("first list_tools call should succeed")?;
    assert_eq!(first.tools.len(), 1);
    assert_eq!(list_calls_total.load(Ordering::SeqCst), 1);

    let second = pool
        .list_tools(None)
        .await
        .context("second list_tools call should succeed")?;
    assert_eq!(second.tools.len(), 1);
    assert_eq!(
        list_calls_total.load(Ordering::SeqCst),
        1,
        "second call should be served from cache"
    );

    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let third = pool
        .list_tools(None)
        .await
        .context("third list_tools call should succeed")?;
    assert_eq!(third.tools.len(), 1);
    assert_eq!(
        list_calls_total.load(Ordering::SeqCst),
        2,
        "cache should refresh after ttl"
    );
    let cache_snapshot = pool.tools_list_cache_stats_snapshot();
    assert_eq!(cache_snapshot.requests_total, 3);
    assert_eq!(cache_snapshot.cache_hits, 1);
    assert_eq!(cache_snapshot.cache_misses, 2);
    assert_eq!(cache_snapshot.cache_refreshes, 2);
    assert!((cache_snapshot.hit_rate_pct - 33.33).abs() < 0.005);

    handle.abort();
    let _ = handle.await;
    Ok(())
}

#[tokio::test]
async fn mcp_pool_list_tools_recovers_after_server_restart() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let handle_1 = spawn_mock_server(addr, 0).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, reconnect_test_config(1), None)
        .await
        .context("connect pool")?;

    let initial = pool
        .list_tools(None)
        .await
        .context("initial list_tools call should succeed")?;
    assert_eq!(initial.tools.len(), 1);

    handle_1.abort();
    let _ = handle_1.await;

    let restart = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        spawn_mock_server(addr, 0).await
    });

    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let recovered = pool
        .list_tools(None)
        .await
        .context("list_tools should recover after reconnect")?;
    assert_eq!(recovered.tools.len(), 1);

    let handle_2 = restart
        .await
        .map_err(|error| anyhow!("restart task join failed: {error}"))??;
    handle_2.abort();
    let _ = handle_2.await;
    Ok(())
}

#[tokio::test]
async fn mcp_pool_call_tool_recovers_after_server_restart() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let handle_1 = spawn_mock_server(addr, 0).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, reconnect_test_config(1), None)
        .await
        .context("connect pool")?;

    let initial = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "first" })),
        )
        .await
        .context("initial call_tool should succeed")?;
    assert_eq!(initial.content.len(), 1);

    handle_1.abort();
    let _ = handle_1.await;

    let restart = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        spawn_mock_server(addr, 0).await
    });

    let recovered = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "second" })),
        )
        .await
        .context("call_tool should recover after reconnect")?;
    assert_eq!(recovered.content.len(), 1);

    let handle_2 = restart
        .await
        .map_err(|error| anyhow!("restart task join failed: {error}"))??;
    handle_2.abort();
    let _ = handle_2.await;
    Ok(())
}

#[tokio::test]
async fn mcp_pool_call_tool_embedding_timeout_is_not_retried_as_transport() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let (handle, call_calls_total) = spawn_embedding_timeout_server(addr).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, reconnect_test_config(1), None)
        .await
        .context("connect pool")?;

    let call_result = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "timeout" })),
        )
        .await;
    let Err(error) = call_result else {
        return Err(anyhow!("embedding timeout should return an error"));
    };
    let message = format!("{error:#}");
    assert!(
        message.to_ascii_lowercase().contains("embedding timed out"),
        "unexpected error message: {message}"
    );
    assert_eq!(
        call_calls_total.load(Ordering::SeqCst),
        1,
        "embedding timeout should not trigger reconnect+retry transport loop"
    );

    handle.abort();
    let _ = handle.await;
    Ok(())
}

#[tokio::test]
async fn mcp_pool_list_tools_falls_back_to_next_client_on_non_transport_error() -> Result<()> {
    let addr = reserve_local_addr().await?;
    let handle = spawn_mock_server(addr, 1).await?;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, reconnect_test_config(2), None)
        .await
        .context("connect pool")?;

    let result = pool
        .list_tools(None)
        .await
        .context("list_tools should fall back to next client")?;
    assert_eq!(result.tools.len(), 1);

    handle.abort();
    let _ = handle.await;
    Ok(())
}
