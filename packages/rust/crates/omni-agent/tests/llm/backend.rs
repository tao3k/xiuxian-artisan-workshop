//! Test coverage for omni-agent behavior.

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
