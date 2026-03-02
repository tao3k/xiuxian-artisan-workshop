//! Tests for `OmniMcpClient`: `from_config`, `list_tools/call_tool` before
//! connect.

use anyhow::{Result, anyhow};
use xiuxian_mcp::{McpServerTransportConfig, OmniMcpClient};

#[test]
fn from_config_streamable_http_creates_client() {
    let config = McpServerTransportConfig::StreamableHttp {
        url: "http://127.0.0.1:3000".to_string(),
        bearer_token_env_var: None,
    };
    let _client = OmniMcpClient::from_config(&config);
}

#[test]
fn from_config_stdio_creates_client() {
    let config = McpServerTransportConfig::Stdio {
        command: "true".to_string(),
        args: vec![],
    };
    let _client = OmniMcpClient::from_config(&config);
}

#[tokio::test]
async fn list_tools_before_connect_returns_error() -> Result<()> {
    let config = McpServerTransportConfig::StreamableHttp {
        url: "http://127.0.0.1:3000".to_string(),
        bearer_token_env_var: None,
    };
    let client = OmniMcpClient::from_config(&config);
    let Err(err) = client.list_tools(None).await else {
        return Err(anyhow!("list_tools should fail before connect"));
    };
    let msg = err.to_string();
    assert!(
        msg.contains("not initialized"),
        "expected 'not initialized', got: {msg}"
    );
    Ok(())
}

#[tokio::test]
async fn call_tool_before_connect_returns_error() -> Result<()> {
    let config = McpServerTransportConfig::StreamableHttp {
        url: "http://127.0.0.1:3000".to_string(),
        bearer_token_env_var: None,
    };
    let client = OmniMcpClient::from_config(&config);
    let err = client
        .call_tool(
            "demo.echo".to_string(),
            Some(serde_json::json!({"message": "hi"})),
        )
        .await;
    let Err(err) = err else {
        return Err(anyhow!("call_tool should fail before connect"));
    };
    let msg = err.to_string();
    assert!(
        msg.contains("not initialized"),
        "expected 'not initialized', got: {msg}"
    );
    Ok(())
}
