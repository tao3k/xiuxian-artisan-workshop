//! Integration tests for wendao domain router mounted into zhenfa gateway.
#![cfg(feature = "zhenfa-router")]

use std::fs;

use serde_json::json;
use tempfile::TempDir;
use xiuxian_wendao::WendaoZhenfaRouter;
use xiuxian_zhenfa::{INVALID_PARAMS_CODE, ZhenfaGatewayBuilder};

async fn spawn_app(
    router: axum::Router,
) -> Result<(String, tokio::task::JoinHandle<()>), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    Ok((format!("http://{addr}"), handle))
}

fn build_notebook_fixture() -> TempDir {
    let temp_dir = TempDir::new().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let alpha_note = temp_dir.path().join("alpha.md");
    let beta_note = temp_dir.path().join("beta.md");
    fs::write(
        &alpha_note,
        "# Router Migration\n\nWendao router should support zhenfa search bridge.\n",
    )
    .unwrap_or_else(|error| panic!("write alpha note: {error}"));
    fs::write(
        &beta_note,
        "# Memory Notes\n\nQianhuan and Wendao are now routed via matrix gateway.\n",
    )
    .unwrap_or_else(|error| panic!("write beta note: {error}"));
    temp_dir
}

#[tokio::test]
async fn wendao_router_rpc_search_returns_markdown_payload() {
    let notebook = build_notebook_fixture();
    let app = ZhenfaGatewayBuilder::new()
        .add_router(WendaoZhenfaRouter::new())
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "req-wendao-search",
            "method": "wendao.search",
            "params": {
                "query": "router",
                "limit": 5,
                "root_dir": notebook.path().to_string_lossy().to_string()
            }
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("rpc request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("rpc payload should parse: {error}"));
    let result = payload["result"]
        .as_str()
        .unwrap_or_else(|| panic!("result should be string payload: {payload}"));
    assert!(result.contains("Wendao Search Results"));
    assert!(result.contains("Router Migration"));

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn wendao_router_http_search_returns_result_field() {
    let notebook = build_notebook_fixture();
    let app = ZhenfaGatewayBuilder::new()
        .add_router(WendaoZhenfaRouter::new())
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/v1/wendao/search"))
        .json(&json!({
            "query": "gateway",
            "limit": 5,
            "root_dir": notebook.path().to_string_lossy().to_string()
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("http search request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("http payload should parse: {error}"));
    let result = payload["result"]
        .as_str()
        .unwrap_or_else(|| panic!("result should be string payload: {payload}"));
    assert!(result.contains("Wendao Search Results"));

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn wendao_router_rpc_search_rejects_invalid_params() {
    let app = ZhenfaGatewayBuilder::new()
        .add_router(WendaoZhenfaRouter::new())
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "req-invalid",
            "method": "wendao.search",
            "params": {
                "limit": 5
            }
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("rpc request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("rpc payload should parse: {error}"));
    assert_eq!(payload["error"]["code"], INVALID_PARAMS_CODE);

    handle.abort();
    let _ = handle.await;
}
