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

//! MCP pool reconnect smoke tests for omni-agent MCP facade.
//!
//! Detailed reconnect/cache/fallback behavior is covered in
//! `xiuxian-llm/tests/mcp_pool_reconnect.rs`.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use omni_agent::{McpPoolConnectConfig, connect_pool};
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

#[derive(Clone, Default)]
struct MockMcpServer;

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
            description: Some("Echo for reconnect smoke test".into()),
            input_schema: Arc::new(map),
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

async fn spawn_mock_server(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    let service: StreamableHttpService<MockMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(MockMcpServer),
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
        .expect("bind mock mcp listener");
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    })
}

fn reconnect_test_config() -> McpPoolConnectConfig {
    McpPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 1,
        connect_retries: 6,
        connect_retry_backoff_ms: 100,
        tool_timeout_secs: 10,
        list_tools_cache_ttl_ms: 1_000,
    }
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve local addr");
    let addr = probe.local_addr().expect("read reserved local addr");
    drop(probe);
    addr
}

#[tokio::test]
async fn mcp_pool_call_tool_recovers_after_server_restart() {
    let addr = reserve_local_addr().await;
    let handle_1 = spawn_mock_server(addr).await;
    let url = format!("http://{addr}/sse");
    let pool = connect_pool(&url, reconnect_test_config())
        .await
        .expect("connect pool");

    let initial = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "first" })),
        )
        .await
        .expect("initial call_tool");
    assert_eq!(initial.content.len(), 1);

    handle_1.abort();
    let _ = handle_1.await;

    let restart = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        spawn_mock_server(addr).await
    });

    let recovered = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "second" })),
        )
        .await
        .expect("call_tool should recover after reconnect");
    assert_eq!(recovered.content.len(), 1);

    let handle_2 = restart.await.expect("restart task join");
    handle_2.abort();
    let _ = handle_2.await;
}
