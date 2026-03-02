//! Integration tests for `xiuxian_wendao::zhenfa_router::rpc`.

#![cfg(feature = "zhenfa-router")]

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use xiuxian_wendao::search_from_rpc_params;
use xiuxian_zhenfa::INTERNAL_ERROR_CODE;

#[test]
fn search_from_rpc_params_rejects_empty_query() {
    let params = json!({
        "query": "   ",
    });

    let error = search_from_rpc_params(params)
        .expect_err("empty query should produce JSON-RPC error payload");
    assert_eq!(error.code, INTERNAL_ERROR_CODE);
    assert_eq!(error.message, "wendao search failed");
}

#[test]
fn search_from_rpc_params_markdown_response_contains_title() {
    let temp_dir = TempDir::new().unwrap_or_else(|error| panic!("temp dir failed: {error}"));
    fs::write(
        temp_dir.path().join("alpha.md"),
        "# Alpha\n\nrouter integration content",
    )
    .unwrap_or_else(|error| panic!("fixture write failed: {error}"));

    let params = json!({
        "query": "router",
        "root_dir": temp_dir.path().to_string_lossy(),
        "response_format": "markdown",
        "limit": 0
    });
    let result = search_from_rpc_params(params)
        .unwrap_or_else(|error| panic!("search_from_rpc_params should succeed: {error:?}"));
    assert!(result.contains("Wendao Search Results"));
}
