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

use super::*;

#[test]
fn parse_backend_mode_defaults_to_litellm_rs() {
    assert_eq!(
        backend::parse_backend_mode(None),
        backend::LlmBackendMode::LiteLlmRs
    );
    assert_eq!(
        backend::parse_backend_mode(Some("")),
        backend::LlmBackendMode::LiteLlmRs
    );
}

#[test]
fn parse_backend_mode_accepts_litellm_rs_aliases() {
    assert_eq!(
        backend::parse_backend_mode(Some("litellm_rs")),
        backend::LlmBackendMode::LiteLlmRs
    );
    assert_eq!(
        backend::parse_backend_mode(Some("litellm-rs")),
        backend::LlmBackendMode::LiteLlmRs
    );
}

#[test]
fn parse_backend_mode_accepts_mistral_aliases() {
    assert_eq!(
        backend::parse_backend_mode(Some("mistral_local")),
        backend::LlmBackendMode::MistralLocal
    );
    assert_eq!(
        backend::parse_backend_mode(Some("mistral-server")),
        backend::LlmBackendMode::MistralLocal
    );
}

#[test]
fn parse_backend_mode_invalid_value_falls_back_to_litellm_rs() {
    assert_eq!(
        backend::parse_backend_mode(Some("unsupported-backend")),
        backend::LlmBackendMode::LiteLlmRs
    );
}

#[test]
fn extract_api_base_from_inference_url_strips_completion_suffix() {
    let base =
        backend::extract_api_base_from_inference_url("http://127.0.0.1:4000/v1/chat/completions");
    assert_eq!(base, "http://127.0.0.1:4000/v1");
}

#[test]
fn parse_tools_json_keeps_name_description_and_schema() {
    let tools = tools::parse_tools_json(Some(vec![serde_json::json!({
        "name": "crawl4ai.crawl_url",
        "description": "crawl web page",
        "input_schema": {"type":"object","properties":{"url":{"type":"string"}}}
    })]));
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "crawl4ai.crawl_url");
    assert_eq!(tools[0].description.as_deref(), Some("crawl web page"));
    assert!(tools[0].parameters.is_some());
}
