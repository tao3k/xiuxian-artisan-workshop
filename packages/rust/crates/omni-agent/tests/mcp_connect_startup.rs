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

    let error = match Agent::from_config(config).await {
        Ok(_) => panic!("startup should fail for unreachable MCP endpoint"),
        Err(error) => error,
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
