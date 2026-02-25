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

//! Unit tests for multiple-MCP tool name qualification and parsing (no network).

use omni_agent::{parse_qualified_tool_name, qualify_tool_name};

#[test]
fn qualify_tool_name_format() {
    assert_eq!(
        qualify_tool_name("omniAgent", "run_terminal_cmd"),
        "mcp__omniAgent__run_terminal_cmd"
    );
    assert_eq!(qualify_tool_name("s1", "tool_a"), "mcp__s1__tool_a");
}

#[test]
fn parse_qualified_tool_name_valid() {
    assert_eq!(
        parse_qualified_tool_name("mcp__omniAgent__run_terminal_cmd"),
        Some(("omniAgent".to_string(), "run_terminal_cmd".to_string()))
    );
    assert_eq!(
        parse_qualified_tool_name("mcp__s1__tool_a"),
        Some(("s1".to_string(), "tool_a".to_string()))
    );
}

#[test]
fn parse_qualified_tool_name_invalid_returns_none() {
    assert!(parse_qualified_tool_name("run_terminal_cmd").is_none());
    assert!(parse_qualified_tool_name("mcp__").is_none());
    assert!(parse_qualified_tool_name("mcp__server_only").is_none());
    assert!(parse_qualified_tool_name("").is_none());
}

#[test]
fn qualify_and_parse_roundtrip() {
    let server = "myServer";
    let tool = "my_tool";
    let qualified = qualify_tool_name(server, tool);
    let parsed = parse_qualified_tool_name(&qualified).unwrap();
    assert_eq!(parsed.0, server);
    assert_eq!(parsed.1, tool);
}
