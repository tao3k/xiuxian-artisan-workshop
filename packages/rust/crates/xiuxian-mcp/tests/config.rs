//! Tests for `McpServerTransportConfig` (de)serialization.

use anyhow::Result;
use xiuxian_mcp::McpServerTransportConfig;

#[test]
fn config_roundtrip_streamable_http() -> Result<()> {
    // Untagged enum: JSON is the variant's fields at top level.
    let json = r#"{"url":"http://127.0.0.1:3000"}"#;
    let config: McpServerTransportConfig = serde_json::from_str(json)?;
    match &config {
        McpServerTransportConfig::StreamableHttp {
            url,
            bearer_token_env_var,
        } => {
            assert_eq!(url, "http://127.0.0.1:3000");
            assert!(bearer_token_env_var.is_none());
        }
        McpServerTransportConfig::Stdio { .. } => panic!("expected StreamableHttp"),
    }
    let out = serde_json::to_string(&config)?;
    let again: McpServerTransportConfig = serde_json::from_str(&out)?;
    match &again {
        McpServerTransportConfig::StreamableHttp { url, .. } => {
            assert_eq!(url, "http://127.0.0.1:3000");
        }
        McpServerTransportConfig::Stdio { .. } => panic!("expected StreamableHttp"),
    }
    Ok(())
}

#[test]
fn config_roundtrip_streamable_http_with_bearer() -> Result<()> {
    let json = r#"{"url":"http://127.0.0.1:3000","bearer_token_env_var":"MCP_TOKEN"}"#;
    let config: McpServerTransportConfig = serde_json::from_str(json)?;
    match &config {
        McpServerTransportConfig::StreamableHttp {
            url,
            bearer_token_env_var,
        } => {
            assert_eq!(url, "http://127.0.0.1:3000");
            assert_eq!(bearer_token_env_var.as_deref(), Some("MCP_TOKEN"));
        }
        McpServerTransportConfig::Stdio { .. } => panic!("expected StreamableHttp"),
    }
    Ok(())
}

#[test]
fn config_roundtrip_stdio() -> Result<()> {
    let json = r#"{"command":"uv","args":["run","omni","mcp","--transport","stdio"]}"#;
    let config: McpServerTransportConfig = serde_json::from_str(json)?;
    match &config {
        McpServerTransportConfig::Stdio { command, args } => {
            assert_eq!(command, "uv");
            assert_eq!(args, &["run", "omni", "mcp", "--transport", "stdio"]);
        }
        McpServerTransportConfig::StreamableHttp { .. } => panic!("expected Stdio"),
    }
    let out = serde_json::to_string(&config)?;
    let again: McpServerTransportConfig = serde_json::from_str(&out)?;
    match &again {
        McpServerTransportConfig::Stdio { command, args } => {
            assert_eq!(command, "uv");
            assert_eq!(args.len(), 5);
        }
        McpServerTransportConfig::StreamableHttp { .. } => panic!("expected Stdio"),
    }
    Ok(())
}

#[test]
fn config_stdio_default_args() -> Result<()> {
    let json = r#"{"command":"npx"}"#;
    let config: McpServerTransportConfig = serde_json::from_str(json)?;
    match &config {
        McpServerTransportConfig::Stdio { command, args } => {
            assert_eq!(command, "npx");
            assert!(args.is_empty());
        }
        McpServerTransportConfig::StreamableHttp { .. } => panic!("expected Stdio"),
    }
    Ok(())
}
