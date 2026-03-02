//! Agent injection tests for context shaping and session flow boundaries.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::{Json, Router, extract::State, routing::post};
use omni_agent::{
    Agent, AgentConfig, McpServerEntry, MemoryConfig, SessionStore, set_config_home_override,
};
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::time::sleep;
use xiuxian_qianhuan::InjectionPolicy;

fn require_ok<T, E>(result: std::result::Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn require_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

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
            title: Some(name.to_string()),
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
            Self::tool("bridge.always_fail", "Always fail tool invocation"),
            Self::tool("bridge.large_payload", "Return very large tool payload"),
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
            .map_or_else(|| serde_json::json!({}), serde_json::Value::Object);

        require_ok(
            self.recorded_arguments.lock(),
            "recorded arguments lock poisoned",
        )
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
            "bridge.always_fail" => std::future::ready(Err(ErrorData::internal_error(
                "forced tool failure for resilience tests",
                None,
            ))),
            "bridge.large_payload" => {
                let size = request
                    .arguments
                    .as_ref()
                    .and_then(|value| value.get("size"))
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|value| usize::try_from(value).ok())
                    .unwrap_or(12_000)
                    .clamp(256, 20_000);
                let payload = "X".repeat(size);
                std::future::ready(Ok(CallToolResult::success(vec![Content::text(payload)])))
            }
            _ => {
                let payload = serde_json::to_string(&args_json)
                    .unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_string());
                std::future::ready(Ok(CallToolResult::success(vec![Content::text(payload)])))
            }
        }
    }
}

#[derive(Clone, Copy)]
enum MockLlmScenario {
    ValidToolArguments,
    MalformedToolArguments,
    ReflectionHintRecovery,
    RoleMixSwitch,
    LargePayloadBudgetPressure,
}

#[derive(Clone)]
struct MockLlmServerState {
    requests: Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    scenario: MockLlmScenario,
    round: Arc<AtomicUsize>,
}

struct MockLlmRequestFacts {
    has_tool_response: bool,
    next_turn_hint: Option<String>,
    latest_user_message: String,
}

fn llm_text_response(content: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "choices": [{
            "message": {
                "content": content
            }
        }]
    }))
}

fn llm_tool_response(name: &str, arguments: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "choices": [{
            "message": {
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments
                    }
                }]
            }
        }]
    }))
}

fn llm_role_mix_normal_response() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "choices": [{
            "message": {
                "content": "role-mix-normal",
                "tool_calls": null
            }
        }]
    }))
}

fn collect_mock_llm_request_facts(payload: &serde_json::Value) -> MockLlmRequestFacts {
    let has_tool_response = payload_messages(payload)
        .iter()
        .any(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("tool"));
    let next_turn_hint = find_message_by_name(payload, "agent.next_turn_hint")
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let latest_user_message = payload_messages(payload)
        .iter()
        .rev()
        .find_map(|message| {
            (message.get("role").and_then(serde_json::Value::as_str) == Some("user")).then(|| {
                message
                    .get("content")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
        })
        .unwrap_or_default();
    MockLlmRequestFacts {
        has_tool_response,
        next_turn_hint,
        latest_user_message,
    }
}

fn mock_llm_scenario_response(
    scenario: MockLlmScenario,
    facts: &MockLlmRequestFacts,
) -> Json<serde_json::Value> {
    match scenario {
        MockLlmScenario::ValidToolArguments => {
            if facts.has_tool_response {
                return llm_text_response("react-ok");
            }
            llm_tool_response("bridge.echo", r#"{"task":"react-loop"}"#)
        }
        MockLlmScenario::MalformedToolArguments => {
            if facts.has_tool_response {
                return llm_text_response("react-ok");
            }
            llm_tool_response("bridge.echo", "{not-json")
        }
        MockLlmScenario::ReflectionHintRecovery => {
            if facts.has_tool_response {
                return llm_text_response("react-ok");
            }
            if facts.next_turn_hint.is_some() {
                return llm_tool_response(
                    "bridge.echo",
                    r#"{"task":"corrected-by-next-turn-hint"}"#,
                );
            }
            llm_tool_response("bridge.always_fail", "{}")
        }
        MockLlmScenario::RoleMixSwitch => {
            if facts.has_tool_response {
                return llm_text_response("role-mix-recovery-ok");
            }
            if let Some(hint) = facts.next_turn_hint.as_deref()
                && hint.contains("role_mix_profile=recovery")
            {
                return llm_tool_response("bridge.echo", r#"{"task":"role-mix-recovery"}"#);
            }
            if facts
                .latest_user_message
                .contains("trigger role mix failure")
            {
                return llm_tool_response("bridge.always_fail", "{}");
            }
            llm_role_mix_normal_response()
        }
        MockLlmScenario::LargePayloadBudgetPressure => {
            if facts.has_tool_response {
                return llm_text_response("budget-ok");
            }
            llm_tool_response("bridge.large_payload", r#"{"size":12000}"#)
        }
    }
}

async fn mock_llm_chat_handler(
    State(state): State<MockLlmServerState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    require_ok(state.requests.lock(), "mock llm requests lock poisoned").push(payload.clone());
    let _round = state.round.fetch_add(1, Ordering::SeqCst);
    let facts = collect_mock_llm_request_facts(&payload);
    mock_llm_scenario_response(state.scenario, &facts)
}

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = require_ok(
        tokio::net::TcpListener::bind("127.0.0.1:0").await,
        "reserve local addr",
    );
    let addr = require_ok(probe.local_addr(), "read reserved local addr");
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
    let listener = require_ok(
        tokio::net::TcpListener::bind(addr).await,
        "bind mock mcp listener",
    );

    (
        tokio::spawn(async move {
            let _ = axum::serve(listener, router).await;
        }),
        recorded_arguments,
    )
}

async fn spawn_mock_llm_server(
    addr: std::net::SocketAddr,
    scenario: MockLlmScenario,
) -> (
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
) {
    let requests = Arc::new(std::sync::Mutex::new(Vec::new()));
    let state = MockLlmServerState {
        requests: Arc::clone(&requests),
        scenario,
        round: Arc::new(AtomicUsize::new(0)),
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(mock_llm_chat_handler))
        .with_state(state);
    let listener = require_ok(
        tokio::net::TcpListener::bind(addr).await,
        "bind mock llm listener",
    );
    (
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        }),
        requests,
    )
}

fn ensure_http_llm_backend_for_tests() {
    static CONFIG_HOME: OnceLock<PathBuf> = OnceLock::new();
    let path = CONFIG_HOME.get_or_init(|| {
        let root = std::env::temp_dir()
            .join("omni-agent-tests")
            .join("agent_injection");
        let settings_dir = root.join("xiuxian-artisan-workshop");
        require_ok(
            std::fs::create_dir_all(&settings_dir),
            "create isolated config home for tests",
        );
        require_ok(
            std::fs::write(
                settings_dir.join("xiuxian.toml"),
                "[agent]\nllm_backend = \"http\"\nagenda_validation_policy = \"never\"\n",
            ),
            "write isolated runtime settings for tests",
        );
        root
    });
    set_config_home_override(path.clone());
}

fn base_config(inference_url: String, mcp_url: String) -> AgentConfig {
    ensure_http_llm_backend_for_tests();
    AgentConfig {
        inference_url,
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

fn payload_messages(payload: &serde_json::Value) -> &[serde_json::Value] {
    payload
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn find_message_by_name<'a>(
    payload: &'a serde_json::Value,
    name: &str,
) -> Option<&'a serde_json::Value> {
    payload_messages(payload)
        .iter()
        .find(|message| message.get("name").and_then(serde_json::Value::as_str) == Some(name))
}

fn has_next_turn_hint(payload: &serde_json::Value) -> bool {
    find_message_by_name(payload, "agent.next_turn_hint").is_some()
}

#[tokio::test]
async fn react_loop_tool_call_roundtrip_with_mock_llm_and_mcp() -> Result<()> {
    let mcp_addr = reserve_local_addr().await;
    let (mcp_server, recorded_arguments) = spawn_mock_bridge_server(mcp_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ValidToolArguments).await;
    let mcp_url = format!("http://{mcp_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let agent = Agent::from_config(base_config(inference_url, mcp_url)).await?;
    let output = agent
        .run_turn(
            "telegram:-100200:42",
            "use bridge.echo to complete the task",
        )
        .await?;
    assert_eq!(output, "react-ok");

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(
        captured.len(),
        1,
        "react flow should issue one MCP tool call"
    );
    assert_eq!(captured[0]["task"], "react-loop");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert_eq!(
        llm_payloads.len(),
        2,
        "react loop should call LLM twice (tool plan + final answer)"
    );
    assert!(
        llm_payloads[0]
            .get("tools")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|tools| !tools.is_empty()),
        "first LLM request should include tool definitions"
    );
    assert!(
        llm_payloads[1]
            .get("messages")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|messages| messages.iter().any(|message| {
                message.get("role").and_then(serde_json::Value::as_str) == Some("tool")
            })),
        "second LLM request should include tool result message"
    );

    mcp_server.abort();
    let _ = mcp_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_shortcut_strips_prefix_before_llm_prompt() -> Result<()> {
    let mcp_addr = reserve_local_addr().await;
    let (mcp_server, _recorded_arguments) = spawn_mock_bridge_server(mcp_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ValidToolArguments).await;
    let mcp_url = format!("http://{mcp_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let agent = Agent::from_config(base_config(inference_url, mcp_url)).await?;
    let output = agent
        .run_turn(
            "telegram:-100300:7",
            "!react call bridge.echo with task react-loop",
        )
        .await?;
    assert_eq!(output, "react-ok");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    let first_messages = llm_payloads[0]
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("first llm payload must include messages"));
    let user_message = first_messages
        .iter()
        .rev()
        .find(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("user"))
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str);
    assert_eq!(
        user_message,
        Some("call bridge.echo with task react-loop"),
        "`!react` prefix should be removed before sending prompt to LLM"
    );

    mcp_server.abort();
    let _ = mcp_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_loop_malformed_tool_arguments_fall_back_to_empty_object() -> Result<()> {
    let mcp_addr = reserve_local_addr().await;
    let (mcp_server, recorded_arguments) = spawn_mock_bridge_server(mcp_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, _llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::MalformedToolArguments).await;
    let mcp_url = format!("http://{mcp_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let mut config = base_config(inference_url, mcp_url);
    let temp_dir = tempfile::tempdir()?;
    config.memory = Some(MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name: "react_malformed_tool_arguments".to_string(),
        persistence_backend: "local".to_string(),
        embedding_base_url: Some("http://127.0.0.1:9".to_string()),
        ..MemoryConfig::default()
    });
    let agent = Agent::from_config(config).await?;

    let output = agent
        .run_turn("telegram:-100300:8", "simulate malformed tool arguments")
        .await?;
    assert_eq!(output, "react-ok");

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(captured.len(), 1, "expected one tool call");
    assert!(
        captured[0]
            .as_object()
            .is_some_and(serde_json::Map::is_empty),
        "invalid JSON tool arguments should degrade to empty object"
    );

    mcp_server.abort();
    let _ = mcp_server.await;
    llm_server.abort();
    let _ = llm_server.await;
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

    let config = base_config(
        "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        mcp_url,
    );
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

    let fields = require_some(fields, "expected route trace stream event in route.events");
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

    let trace_json = require_some(
        fields.get("route_trace_json"),
        "route_trace_json should exist",
    );
    let trace: serde_json::Value = require_ok(
        serde_json::from_str(trace_json),
        "route_trace_json should be valid json",
    );
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

#[tokio::test]
async fn react_failure_reflection_injects_next_turn_hint_and_recovers() -> Result<()> {
    let mcp_addr = reserve_local_addr().await;
    let (mcp_server, recorded_arguments) = spawn_mock_bridge_server(mcp_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ReflectionHintRecovery).await;
    let mcp_url = format!("http://{mcp_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");
    let agent = Agent::from_config(base_config(inference_url, mcp_url)).await?;
    let session_id = "telegram:-100500:10";

    let first_attempt = agent
        .run_turn(session_id, "trigger correction flow with a failing tool")
        .await;
    let Err(first_error) = first_attempt else {
        panic!("first turn should fail and trigger reflection");
    };
    assert!(
        first_error
            .to_string()
            .contains("forced tool failure for resilience tests")
    );

    let output = agent
        .run_turn(session_id, "retry after reflection correction")
        .await?;
    assert_eq!(output, "react-ok");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert_eq!(
        llm_payloads.len(),
        3,
        "expected 1 request on failed turn and 2 requests on recovered turn"
    );
    assert!(
        !has_next_turn_hint(&llm_payloads[0]),
        "first turn must not include next-turn hint before reflection exists"
    );
    assert!(
        has_next_turn_hint(&llm_payloads[1]),
        "second turn should inject stored next-turn hint"
    );
    let hint_message = find_message_by_name(&llm_payloads[1], "agent.next_turn_hint")
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(hint_message.contains("reason=previous_turn_error_requires_verification"));
    assert!(hint_message.contains("role_mix_profile=recovery"));

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(
        captured.len(),
        2,
        "expected two tool invocations (one forced failure + one corrected call)"
    );
    assert_eq!(captured[1]["task"], "corrected-by-next-turn-hint");

    mcp_server.abort();
    let _ = mcp_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_budget_pressure_truncates_tool_payload_and_keeps_core_injection_anchor() -> Result<()>
{
    let mcp_addr = reserve_local_addr().await;
    let (mcp_server, _recorded_arguments) = spawn_mock_bridge_server(mcp_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::LargePayloadBudgetPressure).await;
    let mcp_url = format!("http://{mcp_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let mut config = base_config(inference_url, mcp_url);
    config.context_budget_tokens = Some(140);
    config.context_budget_reserve_tokens = 20;
    let agent = Agent::from_config(config).await?;
    let session_id = "telegram:-100500:11";

    agent
        .upsert_session_system_prompt_injection_xml(
            session_id,
            r"
<system_prompt_injection>
  <qa>
    <q>core anchor</q>
    <a>Keep genesis_rules and persona_steering anchors available under budget pressure.</a>
  </qa>
</system_prompt_injection>
",
        )
        .await?;

    let first = agent
        .run_turn(session_id, "run budget-pressure payload test turn one")
        .await?;
    assert_eq!(first, "budget-ok");

    let second = agent
        .run_turn(session_id, "run budget-pressure payload test turn two")
        .await?;
    assert_eq!(second, "budget-ok");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert!(
        llm_payloads.len() >= 4,
        "expected two turns, each with tool-plan + final-answer calls"
    );

    let tool_message_content = payload_messages(&llm_payloads[1])
        .iter()
        .find(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("tool"))
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(
        tool_message_content.chars().count() <= InjectionPolicy::default().max_chars,
        "tool payload should be truncated by injection policy max_chars"
    );

    let second_turn_first_payload = &llm_payloads[2];
    let injection_message = find_message_by_name(
        second_turn_first_payload,
        "agent.system_prompt.injection.context",
    )
    .and_then(|message| message.get("content"))
    .and_then(serde_json::Value::as_str)
    .unwrap_or_default();
    assert!(
        injection_message.contains("genesis_rules"),
        "core genesis_rules anchor should survive budget pressure"
    );
    assert!(
        injection_message.contains("persona_steering"),
        "core persona_steering anchor should survive budget pressure"
    );

    let snapshot = require_some(
        agent.inspect_context_budget_snapshot(session_id).await,
        "context budget snapshot should be recorded",
    );
    assert!(
        snapshot.pre_tokens > snapshot.post_tokens,
        "budget pressure scenario should drop/truncate context tokens"
    );

    mcp_server.abort();
    let _ = mcp_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_role_mix_switches_from_recovery_back_to_normal_after_failure_cycle() -> Result<()> {
    let mcp_addr = reserve_local_addr().await;
    let (mcp_server, recorded_arguments) = spawn_mock_bridge_server(mcp_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::RoleMixSwitch).await;
    let mcp_url = format!("http://{mcp_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");
    let agent = Agent::from_config(base_config(inference_url, mcp_url)).await?;
    let session_id = "telegram:-100500:12";

    let first_attempt = agent.run_turn(session_id, "trigger role mix failure").await;
    assert!(
        first_attempt.is_err(),
        "first turn should fail and arm recovery role mix"
    );

    let recovered = agent
        .run_turn(session_id, "second turn should enter recovery profile")
        .await?;
    assert_eq!(recovered, "role-mix-recovery-ok");

    let normal = agent
        .run_turn(session_id, "third turn should return to normal profile")
        .await?;
    assert_eq!(normal, "role-mix-normal");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert!(
        llm_payloads.len() >= 4,
        "expected failure turn + recovery turn + normal turn request sequence"
    );

    let recovery_hint = find_message_by_name(&llm_payloads[1], "agent.next_turn_hint")
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(
        recovery_hint.contains("role_mix_profile=recovery"),
        "failure-following turn should switch into recovery role mix"
    );
    assert!(
        !has_next_turn_hint(&llm_payloads[3]),
        "subsequent normal turn should not keep stale recovery hint"
    );

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(
        captured.len(),
        2,
        "recovery cycle should invoke failing tool once and recovery tool once"
    );
    assert_eq!(captured[1]["task"], "role-mix-recovery");

    mcp_server.abort();
    let _ = mcp_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}
