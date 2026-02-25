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

//! Tests for MCP config loading (mcp.json only, no env fallback).

use omni_agent::load_mcp_config;
use std::io::Write;

#[test]
fn load_mcp_config_missing_file_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");
    let servers = load_mcp_config(&path).unwrap();
    assert!(servers.is_empty());
}

#[test]
fn load_mcp_config_http_server_preserves_base_url() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    let json = r#"{"mcpServers":{"omniAgent":{"type":"http","url":"http://127.0.0.1:3002"}}}"#;
    std::fs::File::create(&path)
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    let servers = load_mcp_config(&path).unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "omniAgent");
    assert_eq!(
        servers[0].url.as_deref(),
        Some("http://127.0.0.1:3002"),
        "HTTP URL must be preserved to avoid forcing a legacy MCP route"
    );
    assert!(servers[0].command.is_none());
}

#[test]
fn load_mcp_config_http_server_preserves_existing_sse() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    let json = r#"{"mcpServers":{"omniAgent":{"type":"http","url":"http://127.0.0.1:3002/sse"}}}"#;
    std::fs::File::create(&path)
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    let servers = load_mcp_config(&path).unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].url.as_deref(), Some("http://127.0.0.1:3002/sse"));
}

#[test]
fn load_mcp_config_http_server_trims_messages_trailing_slash() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    let json =
        r#"{"mcpServers":{"omniAgent":{"type":"http","url":"http://127.0.0.1:3002/messages/"}}}"#;
    std::fs::File::create(&path)
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    let servers = load_mcp_config(&path).unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(
        servers[0].url.as_deref(),
        Some("http://127.0.0.1:3002/messages")
    );
}

#[test]
fn load_mcp_config_stdio_server() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mcp.json");
    let json = r#"{"mcpServers":{"stdioAgent":{"type":"stdio","command":"omni","args":["mcp","--transport","stdio"]}}}"#;
    std::fs::File::create(&path)
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();
    let servers = load_mcp_config(&path).unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "stdioAgent");
    assert!(servers[0].url.is_none());
    assert_eq!(servers[0].command.as_deref(), Some("omni"));
    assert_eq!(
        servers[0].args.as_ref().map(|a| a.as_slice()),
        Some(
            &[
                "mcp".to_string(),
                "--transport".to_string(),
                "stdio".to_string()
            ][..]
        )
    );
}
