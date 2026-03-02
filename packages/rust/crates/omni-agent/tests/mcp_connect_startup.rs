//! Test coverage for omni-agent behavior.

//! Startup MCP connect behavior.

use omni_agent::{Agent, AgentConfig, McpServerEntry};

#[tokio::test]
async fn agent_startup_mcp_connect_retries_are_applied() {
    let config = AgentConfig {
        mcp_servers: vec![McpServerEntry {
            name: "local-unreachable".to_string(),
            url: Some("http://127.0.0.1:1/sse".to_string()),
            command: None,
            args: None,
        }],
        mcp_pool_size: 1,
        mcp_handshake_timeout_secs: 1,
        mcp_connect_retries: 2,
        mcp_strict_startup: true,
        mcp_connect_retry_backoff_ms: 10,
        ..Default::default()
    };

    let Err(error) = Agent::from_config(config).await else {
        panic!("startup should fail for unreachable MCP endpoint");
    };
    let message = format!("{error:#}");
    assert!(
        message.contains("MCP connect failed after 2 attempts"),
        "unexpected error message: {message}"
    );
    assert!(
        message.contains("http://127.0.0.1:1/sse"),
        "unexpected error message: {message}"
    );
}

#[tokio::test]
async fn agent_startup_non_strict_mcp_connect_failure_continues() {
    let config = AgentConfig {
        mcp_servers: vec![McpServerEntry {
            name: "local-unreachable".to_string(),
            url: Some("http://127.0.0.1:1/sse".to_string()),
            command: None,
            args: None,
        }],
        mcp_pool_size: 1,
        mcp_handshake_timeout_secs: 30,
        mcp_connect_retries: 3,
        mcp_strict_startup: false,
        mcp_connect_retry_backoff_ms: 1_000,
        ..Default::default()
    };

    let built = Agent::from_config(config).await;
    assert!(
        built.is_ok(),
        "non-strict startup should continue when MCP is unavailable"
    );
}
