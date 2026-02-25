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

use crate::config::AgentConfig;

use super::startup_connect_config;

#[test]
fn startup_connect_config_keeps_runtime_values_in_strict_mode() {
    let config = AgentConfig {
        mcp_pool_size: 8,
        mcp_handshake_timeout_secs: 45,
        mcp_connect_retries: 4,
        mcp_connect_retry_backoff_ms: 2_000,
        mcp_tool_timeout_secs: 90,
        mcp_list_tools_cache_ttl_ms: 2_500,
        ..Default::default()
    };

    let connect = startup_connect_config(&config, true);
    assert_eq!(connect.pool_size, 8);
    assert_eq!(connect.handshake_timeout_secs, 45);
    assert_eq!(connect.connect_retries, 4);
    assert_eq!(connect.connect_retry_backoff_ms, 2_000);
    assert_eq!(connect.tool_timeout_secs, 90);
    assert_eq!(connect.list_tools_cache_ttl_ms, 2_500);
}

#[test]
fn startup_connect_config_clamps_for_non_strict_mode() {
    let config = AgentConfig {
        mcp_pool_size: 4,
        mcp_handshake_timeout_secs: 120,
        mcp_connect_retries: 9,
        mcp_connect_retry_backoff_ms: 0,
        mcp_tool_timeout_secs: 180,
        mcp_list_tools_cache_ttl_ms: 1_000,
        ..Default::default()
    };

    let connect = startup_connect_config(&config, false);
    assert_eq!(connect.pool_size, 4);
    assert_eq!(connect.handshake_timeout_secs, 5);
    assert_eq!(connect.connect_retries, 1);
    assert_eq!(connect.connect_retry_backoff_ms, 1);
    assert_eq!(connect.tool_timeout_secs, 180);
    assert_eq!(connect.list_tools_cache_ttl_ms, 1_000);
}
