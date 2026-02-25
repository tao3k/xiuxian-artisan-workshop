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

use omni_agent::{GraphBridgeRequest, validate_graph_bridge_request};
use serde_json::json;

#[test]
fn graph_bridge_rejects_empty_tool_name() {
    let request = GraphBridgeRequest {
        tool_name: "   ".to_string(),
        arguments: Some(json!({"query": "x"})),
    };
    let error = validate_graph_bridge_request(&request)
        .expect_err("empty tool name should fail validation");
    assert!(error.to_string().contains("tool_name"));
}

#[test]
fn graph_bridge_rejects_non_object_arguments() {
    let request = GraphBridgeRequest {
        tool_name: "researcher.run_research_graph".to_string(),
        arguments: Some(json!(["not", "an", "object"])),
    };
    let error = validate_graph_bridge_request(&request)
        .expect_err("non-object args should fail validation");
    assert!(error.to_string().contains("JSON object"));
}

#[test]
fn graph_bridge_request_serialization_contract_is_stable() {
    let request = GraphBridgeRequest {
        tool_name: "researcher.run_research_graph".to_string(),
        arguments: Some(json!({
            "repo_url": "https://github.com/example/project",
            "focus": ["architecture", "performance"]
        })),
    };

    let serialized = serde_json::to_value(&request).expect("serialize request");
    let expected = json!({
        "tool_name": "researcher.run_research_graph",
        "arguments": {
            "repo_url": "https://github.com/example/project",
            "focus": ["architecture", "performance"]
        }
    });
    assert_eq!(serialized, expected);
}
