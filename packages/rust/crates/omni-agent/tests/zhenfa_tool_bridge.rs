//! End-to-end tests for native zhenfa tool bridge integration in `omni-agent`.

use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Json, Router, extract::State, routing::post};
use omni_agent::{Agent, AgentConfig, McpServerEntry, set_config_home_override};
use serde_json::json;

#[derive(Clone)]
struct MockLlmScenario {
    tool_name: String,
    tool_arguments: String,
    final_response: String,
}

#[derive(Clone)]
struct MockLlmState {
    round: Arc<AtomicUsize>,
    last_tool_payload: Arc<std::sync::Mutex<Option<String>>>,
    scenario: MockLlmScenario,
}

fn test_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn test_config_root() -> &'static std::path::PathBuf {
    static ROOT: OnceLock<std::path::PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir()
            .join("omni-agent-tests")
            .join(format!("zhenfa-tool-bridge-{}", std::process::id()));
        std::fs::create_dir_all(root.join("xiuxian-artisan-workshop"))
            .unwrap_or_else(|error| panic!("create test config root: {error}"));
        set_config_home_override(root.clone());
        root
    })
}

async fn mock_llm_chat_handler(
    State(state): State<MockLlmState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let round = state.round.fetch_add(1, Ordering::SeqCst);
    if round > 0 {
        let tool_payload = payload
            .get("messages")
            .and_then(serde_json::Value::as_array)
            .and_then(|messages| {
                messages.iter().find_map(|message| {
                    if message.get("role").and_then(serde_json::Value::as_str) != Some("tool") {
                        return None;
                    }
                    Some(extract_message_content(message))
                })
            });
        let mut guard = state
            .last_tool_payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = tool_payload;
    }

    let response = if round == 0 {
        json!({
            "choices": [{
                "message": {
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": state.scenario.tool_name,
                            "arguments": state.scenario.tool_arguments
                        }
                    }]
                }
            }]
        })
    } else {
        json!({
            "choices": [{
                "message": {
                    "content": state.scenario.final_response
                }
            }]
        })
    };
    Json(response)
}

fn extract_message_content(message: &serde_json::Value) -> String {
    if let Some(text) = message.get("content").and_then(serde_json::Value::as_str) {
        return text.to_string();
    }
    message
        .get("content")
        .map_or_else(String::new, serde_json::Value::to_string)
}

async fn reserve_local_addr() -> anyhow::Result<std::net::SocketAddr> {
    let probe = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = probe.local_addr()?;
    drop(probe);
    Ok(addr)
}

async fn spawn_mock_llm_server(
    addr: std::net::SocketAddr,
    scenario: MockLlmScenario,
) -> anyhow::Result<(
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Option<String>>>,
)> {
    let last_tool_payload = Arc::new(std::sync::Mutex::new(None));
    let state = MockLlmState {
        round: Arc::new(AtomicUsize::new(0)),
        last_tool_payload: Arc::clone(&last_tool_payload),
        scenario,
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(mock_llm_chat_handler))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok((
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        }),
        last_tool_payload,
    ))
}

fn write_test_xiuxian_config(
    root: &Path,
    enabled_tools: &[&str],
    notebook_path: Option<&Path>,
) -> anyhow::Result<()> {
    let config_dir = root.join("xiuxian-artisan-workshop");
    std::fs::create_dir_all(&config_dir)?;
    let enabled_tools_toml = enabled_tools
        .iter()
        .map(|tool| format!("\"{tool}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let notebook_toml = notebook_path.map_or_else(String::new, |path| {
        format!(
            "\n[wendao.zhixing]\nnotebook_path = \"{}\"\n",
            path.display()
        )
    });
    std::fs::write(
        config_dir.join("xiuxian.toml"),
        format!(
            r#"[agent]
llm_backend = "http"
agenda_validation_policy = "never"

[zhenfa]
enabled_tools = [{enabled_tools_toml}]
{notebook_toml}
"#
        ),
    )?;
    Ok(())
}

#[tokio::test]
async fn run_turn_dispatches_wendao_search_through_native_zhenfa_bridge() -> anyhow::Result<()> {
    let _guard = test_lock().lock().await;

    let notebook = tempfile::tempdir()?;
    std::fs::write(
        notebook.path().join("alpha.md"),
        "# Native Bridge\n\nWendao native bridge smoke.\n",
    )?;

    let llm_addr = reserve_local_addr().await?;
    let scenario = MockLlmScenario {
        tool_name: "wendao.search".to_string(),
        tool_arguments: format!(
            "{{\"query\":\"native bridge\",\"root_dir\":\"{}\"}}",
            notebook.path().display()
        ),
        final_response: "final answer after native zhenfa bridge call".to_string(),
    };
    let (llm_server, last_tool_payload) = spawn_mock_llm_server(llm_addr, scenario).await?;

    write_test_xiuxian_config(
        test_config_root(),
        &["wendao.search"],
        Some(notebook.path()),
    )?;

    let agent = Agent::from_config(AgentConfig {
        inference_url: format!("http://{llm_addr}/v1/chat/completions"),
        model: "test-model".to_string(),
        mcp_servers: Vec::<McpServerEntry>::new(),
        max_tool_rounds: 4,
        ..AgentConfig::default()
    })
    .await?;

    let output = agent
        .run_turn("telegram:bridge", "search native bridge")
        .await?;
    assert_eq!(output, "final answer after native zhenfa bridge call");

    let seen_tool_payload = last_tool_payload
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_default();
    assert!(
        seen_tool_payload.contains("<hit id=\"alpha.md\""),
        "expected wendao stripped payload in llm second round, got: {seen_tool_payload}"
    );

    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn run_turn_dispatches_qianhuan_reload_through_native_zhenfa_bridge() -> anyhow::Result<()> {
    let _guard = test_lock().lock().await;

    let llm_addr = reserve_local_addr().await?;
    let scenario = MockLlmScenario {
        tool_name: "qianhuan.reload".to_string(),
        tool_arguments: "{}".to_string(),
        final_response: "qianhuan reload completed".to_string(),
    };
    let (llm_server, last_tool_payload) = spawn_mock_llm_server(llm_addr, scenario).await?;

    write_test_xiuxian_config(test_config_root(), &["qianhuan.reload"], None)?;

    let agent = Agent::from_config(AgentConfig {
        inference_url: format!("http://{llm_addr}/v1/chat/completions"),
        model: "test-model".to_string(),
        mcp_servers: Vec::<McpServerEntry>::new(),
        max_tool_rounds: 4,
        ..AgentConfig::default()
    })
    .await?;

    let output = agent
        .run_turn("telegram:bridge", "please reload templates")
        .await?;
    assert_eq!(output, "qianhuan reload completed");

    let seen_tool_payload = last_tool_payload
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_default();
    assert!(
        seen_tool_payload.contains("<qianhuan_reload"),
        "expected qianhuan reload stripped payload in llm second round, got: {seen_tool_payload}"
    );

    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}
