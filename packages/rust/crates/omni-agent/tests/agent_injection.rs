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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::Router;
use omni_agent::{Agent, AgentConfig, McpServerEntry, MemoryConfig, SessionStore};
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::time::sleep;

#[derive(Clone)]
struct MockBridgeServer {
    recorded_arguments: Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    reject_metadata_once_for_flaky: Arc<AtomicBool>,
}

impl MockBridgeServer {
    fn tool(name: &str, description: &str) -> Tool {
        let input_schema = serde_json::json!({
            "type": "object",
            "additionalProperties": true,
        });
        let map = input_schema.as_object().cloned().unwrap_or_default();
        Tool {
            name: name.to_string().into(),
            title: Some(name.to_string().into()),
            description: Some(description.to_string().into()),
            input_schema: Arc::new(map),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        }
    }
}

impl ServerHandler for MockBridgeServer {
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
            Self::tool("bridge.echo", "Echo JSON arguments"),
            Self::tool("bridge.flaky", "Reject first metadata-rich call"),
        ])))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let args_json = request
            .arguments
            .clone()
            .map(serde_json::Value::Object)
            .unwrap_or_else(|| serde_json::json!({}));

        self.recorded_arguments
            .lock()
            .expect("recorded arguments lock poisoned")
            .push(args_json.clone());

        match request.name.as_ref() {
            "bridge.flaky" => {
                let has_metadata = request
                    .arguments
                    .as_ref()
                    .and_then(|value| value.get("_omni"))
                    .is_some();
                if has_metadata
                    && self
                        .reject_metadata_once_for_flaky
                        .swap(false, Ordering::SeqCst)
                {
                    return std::future::ready(Err(ErrorData::internal_error(
                        "metadata not accepted for first attempt",
                        None,
                    )));
                }
                std::future::ready(Ok(CallToolResult::success(vec![Content::text(
                    "fallback-ok".to_string(),
                )])))
            }
            _ => {
                let payload = serde_json::to_string(&args_json)
                    .unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_string());
                std::future::ready(Ok(CallToolResult::success(vec![Content::text(payload)])))
            }
        }
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

async fn spawn_mock_bridge_server(
    addr: std::net::SocketAddr,
) -> (
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
) {
    let recorded_arguments = Arc::new(std::sync::Mutex::new(Vec::new()));
    let reject_metadata_once_for_flaky = Arc::new(AtomicBool::new(true));

    let service: StreamableHttpService<MockBridgeServer, LocalSessionManager> =
        StreamableHttpService::new(
            {
                let recorded_arguments = recorded_arguments.clone();
                let reject_metadata_once_for_flaky = reject_metadata_once_for_flaky.clone();
                move || {
                    Ok(MockBridgeServer {
                        recorded_arguments: recorded_arguments.clone(),
                        reject_metadata_once_for_flaky: reject_metadata_once_for_flaky.clone(),
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
        .expect("bind mock mcp listener");

    (
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        }),
        recorded_arguments,
    )
}

fn base_config(mcp_url: String) -> AgentConfig {
    AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        model: "test-model".to_string(),
        mcp_servers: vec![McpServerEntry {
            name: "mock".to_string(),
            url: Some(mcp_url),
            command: None,
            args: None,
        }],
        mcp_handshake_timeout_secs: 2,
        mcp_connect_retries: 2,
        mcp_connect_retry_backoff_ms: 50,
        mcp_tool_timeout_secs: 15,
        mcp_list_tools_cache_ttl_ms: 100,
        max_tool_rounds: 3,
        ..AgentConfig::default()
    }
}

fn live_redis_url() -> Option<String> {
    for key in ["VALKEY_URL"] {
        if let Ok(url) = std::env::var(key)
            && !url.trim().is_empty()
        {
            return Some(url);
        }
    }
    None
}

fn unique_key_prefix(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

async fn latest_stream_event_fields(
    redis_url: &str,
    key_prefix: &str,
    stream_name: &str,
) -> Result<Option<std::collections::HashMap<String, String>>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let stream_key = format!("{key_prefix}:stream:{stream_name}");
    let entries: Vec<(String, std::collections::HashMap<String, String>)> = redis::cmd("XREVRANGE")
        .arg(&stream_key)
        .arg("+")
        .arg("-")
        .arg("COUNT")
        .arg(1)
        .query_async(&mut conn)
        .await?;
    Ok(entries.into_iter().next().map(|(_, fields)| fields))
}

#[tokio::test]
async fn graph_shortcut_includes_typed_injection_snapshot_metadata() -> Result<()> {
    let addr = reserve_local_addr().await;
    let (server_handle, recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let mcp_url = format!("http://{addr}/sse");

    let temp_dir = tempfile::tempdir()?;
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name: "agent_injection_snapshot".to_string(),
        persistence_backend: "local".to_string(),
        embedding_base_url: Some("http://127.0.0.1:9".to_string()),
        ..MemoryConfig::default()
    };

    let mut config = base_config(mcp_url);
    config.memory = Some(memory);
    config.window_max_turns = Some(1);
    config.consolidation_threshold_turns = Some(1);
    config.consolidation_take_turns = 1;
    config.consolidation_async = false;
    config.summary_max_segments = 32;

    let agent = Agent::from_config(config).await?;
    let session_id = "telegram:-100200:42";

    for index in 0..18 {
        agent
            .append_turn_for_session(
                session_id,
                &format!("historical question {index}"),
                &format!("historical answer {index}"),
            )
            .await?;
    }

    let huge_answer = "X".repeat(12_000);
    let xml =
        format!("<qa><q>critical runtime policy</q><a>{huge_answer}</a><source>ops</source></qa>");
    let _ = agent
        .upsert_session_system_prompt_injection_xml(session_id, &xml)
        .await?;

    let output = agent
        .run_turn(session_id, r#"graph bridge.echo {"task":"snapshot-test"}"#)
        .await?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["task"], "snapshot-test");

    let metadata = payload
        .get("_omni")
        .and_then(serde_json::Value::as_object)
        .expect("shortcut metadata should be attached under _omni");
    assert_eq!(
        metadata
            .get("workflow_mode")
            .and_then(serde_json::Value::as_str),
        Some("graph")
    );
    let graph_plan = metadata
        .get("graph_plan")
        .and_then(serde_json::Value::as_object)
        .expect("graph plan metadata should exist");
    assert_eq!(
        graph_plan
            .get("plan_version")
            .and_then(serde_json::Value::as_str),
        Some("v1")
    );
    assert_eq!(
        graph_plan
            .get("plan_id")
            .and_then(serde_json::Value::as_str),
        Some("graph-plan:graph:bridge.echo:abort:evidence")
    );
    assert_eq!(
        graph_plan.get("route").and_then(serde_json::Value::as_str),
        Some("graph")
    );
    assert_eq!(
        graph_plan
            .get("workflow_mode")
            .and_then(serde_json::Value::as_str),
        Some("graph")
    );
    assert_eq!(
        graph_plan
            .get("tool_name")
            .and_then(serde_json::Value::as_str),
        Some("bridge.echo")
    );
    assert_eq!(
        graph_plan
            .get("fallback_policy")
            .and_then(serde_json::Value::as_str),
        Some("abort")
    );
    let graph_steps = graph_plan
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .expect("graph plan should include deterministic steps");
    assert_eq!(
        graph_steps.len(),
        3,
        "graph plan should have exactly 3 steps"
    );
    assert_eq!(
        graph_steps[0]
            .get("kind")
            .and_then(serde_json::Value::as_str),
        Some("prepare_injection_context")
    );
    assert_eq!(
        graph_steps[1]
            .get("kind")
            .and_then(serde_json::Value::as_str),
        Some("invoke_graph_tool")
    );
    assert_eq!(
        graph_steps[2]
            .get("kind")
            .and_then(serde_json::Value::as_str),
        Some("evaluate_fallback")
    );
    assert_eq!(
        graph_steps[2]
            .get("fallback_action")
            .and_then(serde_json::Value::as_str),
        Some("abort")
    );

    let session_context = metadata
        .get("session_context")
        .and_then(serde_json::Value::as_object)
        .expect("session context metadata should exist");
    assert!(
        session_context
            .get("snapshot_id")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "snapshot_id must exist"
    );
    assert_eq!(
        session_context
            .get("injection_mode")
            .and_then(serde_json::Value::as_str),
        Some("hybrid"),
        "adaptive injection policy should select hybrid mode for multi-domain shortcut context"
    );
    assert!(
        session_context
            .get("dropped_block_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| !items.is_empty()),
        "expected dropped blocks when summary segments exceed max_blocks"
    );
    assert!(
        session_context
            .get("truncated_block_ids")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| !items.is_empty()),
        "expected at least one truncated block when payload exceeds max_chars"
    );
    assert!(
        session_context
            .get("role_mix_profile_id")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "role-mix profile must be attached for multi-domain shortcut injection"
    );
    assert!(
        session_context
            .get("role_mix_roles")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| !items.is_empty()),
        "role-mix role list must be present in shortcut metadata"
    );

    let captured = recorded_arguments
        .lock()
        .expect("recorded arguments lock poisoned")
        .clone();
    assert!(
        !captured.is_empty(),
        "mock MCP should capture at least one tool call"
    );

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn omega_shortcut_retries_without_metadata_after_bridge_error() -> Result<()> {
    let addr = reserve_local_addr().await;
    let (server_handle, recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let mcp_url = format!("http://{addr}/sse");

    let config = base_config(mcp_url);
    let agent = Agent::from_config(config).await?;

    let output = agent
        .run_turn(
            "telegram:-100300:7",
            r#"omega bridge.flaky {"task":"fallback-check"}"#,
        )
        .await?;

    assert_eq!(output, "fallback-ok");

    let captured = recorded_arguments
        .lock()
        .expect("recorded arguments lock poisoned")
        .clone();
    assert_eq!(
        captured.len(),
        2,
        "omega fallback should perform exactly two attempts"
    );

    assert!(
        captured[0].get("_omni").is_some(),
        "first attempt should include omega metadata"
    );
    assert!(
        captured[1].get("_omni").is_none(),
        "fallback attempt should strip metadata for compatibility"
    );

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn omega_shortcut_fallback_uses_real_tool_attempt_count_for_memory_gate() -> Result<()> {
    let addr = reserve_local_addr().await;
    let (server_handle, _recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let mcp_url = format!("http://{addr}/sse");

    let temp_dir = tempfile::tempdir()?;
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name: "omega_shortcut_fallback_attempts".to_string(),
        persistence_backend: "local".to_string(),
        embedding_base_url: Some("http://127.0.0.1:9".to_string()),
        gate_obsolete_threshold: 0.71,
        gate_obsolete_min_usage: 1,
        gate_obsolete_failure_rate_floor: 0.0,
        gate_obsolete_max_ttl_score: 1.0,
        ..MemoryConfig::default()
    };

    let mut config = base_config(mcp_url);
    config.memory = Some(memory);
    let agent = Agent::from_config(config).await?;

    let output = agent
        .run_turn(
            "telegram:-100300:8",
            r#"omega bridge.flaky {"task":"fallback-memory-gate"}"#,
        )
        .await?;
    assert_eq!(output, "fallback-ok");

    let status = agent.inspect_memory_runtime_status();
    assert_eq!(
        status.episodes_total,
        Some(1),
        "successful fallback should keep memory episode when real tool attempts are accounted for"
    );
    assert_eq!(status.q_values_total, Some(1));

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires live valkey server (VALKEY_URL)"]
async fn graph_shortcut_publishes_route_trace_to_route_events_stream() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        return Ok(());
    };

    let addr = reserve_local_addr().await;
    let (server_handle, _recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let mcp_url = format!("http://{addr}/sse");
    let key_prefix = unique_key_prefix("omni-agent-route-trace");

    let config = base_config(mcp_url);
    let session =
        SessionStore::new_with_redis(redis_url.clone(), Some(key_prefix.clone()), Some(120))?;
    let agent = Agent::from_config_with_session_backends_for_test(config, session, None).await?;
    let session_id = "telegram:-100400:11";

    let output = agent
        .run_turn(
            session_id,
            r#"graph bridge.echo {"task":"route-trace-stream"}"#,
        )
        .await?;
    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["task"], "route-trace-stream");

    let mut fields = None;
    for _ in 0..30 {
        fields = latest_stream_event_fields(&redis_url, &key_prefix, "route.events").await?;
        if fields.is_some() {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    let fields = fields.expect("expected route trace stream event in route.events");
    assert_eq!(
        fields.get("kind").map(String::as_str),
        Some("session.route.trace_emitted")
    );
    assert_eq!(
        fields.get("session_id").map(String::as_str),
        Some(session_id)
    );
    assert_eq!(
        fields.get("selected_route").map(String::as_str),
        Some("graph")
    );
    assert_eq!(
        fields.get("workflow_mode").map(String::as_str),
        Some("graph")
    );
    assert_eq!(
        fields.get("graph_steps_count").map(String::as_str),
        Some("3")
    );
    assert!(
        fields
            .get("plan_id")
            .is_some_and(|value| !value.trim().is_empty()),
        "plan_id should be persisted for graph route trace stream events"
    );
    assert!(
        fields
            .get("graph_steps_json")
            .is_some_and(|value| value.contains("invoke_graph_tool")),
        "graph_steps_json should include invoke_graph_tool step"
    );

    let trace_json = fields
        .get("route_trace_json")
        .expect("route_trace_json should exist");
    let trace: serde_json::Value =
        serde_json::from_str(trace_json).expect("route_trace_json should be valid json");
    assert_eq!(trace["selected_route"], "graph");
    assert_eq!(trace["workflow_mode"], "graph");
    assert_eq!(trace["session_id"], session_id);
    assert_eq!(trace["graph_steps"].as_array().map(Vec::len), Some(3));

    let stream_key = format!("{key_prefix}:stream:route.events");
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = redis::cmd("DEL")
        .arg(stream_key)
        .query_async(&mut conn)
        .await?;

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}
