//! Integration tests for qianhuan domain router mounted into zhenfa gateway.
#![cfg(feature = "zhenfa-router")]

use std::sync::Arc;

use serde_json::json;
use xiuxian_qianhuan::{ManifestationManager, QianhuanZhenfaRouter};
use xiuxian_zhenfa::{INVALID_PARAMS_CODE, ZhenfaGatewayBuilder};

fn test_manager() -> ManifestationManager {
    ManifestationManager::new_with_embedded_templates(
        &[],
        &[(
            "daily_agenda.md",
            "Task: {{ task }}\nState: {{ qianhuan.state_context | default(value=\"\") }}",
        )],
    )
    .unwrap_or_else(|error| panic!("create manifestation manager for tests: {error}"))
}

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

#[tokio::test]
async fn qianhuan_router_rpc_render_returns_manifested_payload() {
    let router = QianhuanZhenfaRouter::new(Arc::new(test_manager()));
    let app = ZhenfaGatewayBuilder::new()
        .add_router(router)
        .build()
        .unwrap_or_else(|error| panic!("build zhenfa gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "req-qh-render",
            "method": "qianhuan.render",
            "params": {
                "target": "daily_agenda",
                "data": { "task": "refactor zhenfa bridge" },
                "runtime": { "state_context": "SUCCESS_STREAK" }
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
    assert!(result.contains("Task: refactor zhenfa bridge"));
    assert!(result.contains("State: SUCCESS_STREAK"));

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn qianhuan_router_http_reload_endpoint_is_available() {
    let router = QianhuanZhenfaRouter::new(Arc::new(test_manager()));
    let app = ZhenfaGatewayBuilder::new()
        .add_router(router)
        .build()
        .unwrap_or_else(|error| panic!("build zhenfa gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/v1/qianhuan/reload"))
        .send()
        .await
        .unwrap_or_else(|error| panic!("reload endpoint should respond: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("reload payload should parse: {error}"));
    assert!(payload["reloaded"].is_boolean());

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn qianhuan_router_rpc_render_rejects_invalid_params() {
    let router = QianhuanZhenfaRouter::new(Arc::new(test_manager()));
    let app = ZhenfaGatewayBuilder::new()
        .add_router(router)
        .build()
        .unwrap_or_else(|error| panic!("build zhenfa gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "req-qh-invalid",
            "method": "qianhuan.render",
            "params": { "data": { "task": "missing target field" } }
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
