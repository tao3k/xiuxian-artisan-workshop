//! Integration tests for zhenfa gateway HTTP and JSON-RPC endpoints.

use axum::Router;
use axum::routing::get;
use serde_json::json;
use xiuxian_zhenfa::{
    INVALID_REQUEST_CODE, METHOD_NOT_FOUND_CODE, MethodRegistry, PARSE_ERROR_CODE,
    ZhenfaGatewayBuilder, ZhenfaRouter,
};

struct PingRouter;

impl ZhenfaRouter for PingRouter {
    fn prefix(&self) -> &'static str {
        "/v1/ping"
    }

    fn mount(&self, router: Router) -> Router {
        router.route("/v1/ping/echo", get(|| async { "pong" }))
    }

    fn register_methods(&self, registry: &mut MethodRegistry) {
        registry.register_fn("ping.echo", |_params, _meta| async {
            Ok("pong".to_string())
        });
    }
}

struct BadPrefixRouter;

impl ZhenfaRouter for BadPrefixRouter {
    fn prefix(&self) -> &'static str {
        "v1/bad"
    }

    fn mount(&self, router: Router) -> Router {
        router
    }
}

async fn spawn_app(router: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap_or_else(|error| panic!("bind test listener: {error}"));
    let addr = listener
        .local_addr()
        .unwrap_or_else(|error| panic!("read test listener addr: {error}"));
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    (format!("http://{addr}"), handle)
}

#[tokio::test]
async fn gateway_health_endpoint_returns_ok() {
    let app = ZhenfaGatewayBuilder::new()
        .build()
        .unwrap_or_else(|error| panic!("build empty gateway: {error}"));
    let (base_url, handle) = spawn_app(app).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base_url}/healthz"))
        .send()
        .await
        .unwrap_or_else(|error| panic!("health request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("health response should be json: {error}"));
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["service"], "xiuxian-zhenfa");

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn gateway_rpc_dispatches_registered_method() {
    let app = ZhenfaGatewayBuilder::new()
        .register_method_fn("wendao.search", |params, _meta| async move {
            let query = params
                .get("query")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            Ok(format!("<search>{query}</search>"))
        })
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "wendao.search",
            "id": "req-123",
            "params": { "query": "agenda date:this_week" }
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("rpc request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("rpc response should parse: {error}"));

    assert_eq!(payload["jsonrpc"], "2.0");
    assert_eq!(payload["id"], "req-123");
    assert_eq!(payload["result"], "<search>agenda date:this_week</search>");
    assert!(payload.get("error").is_none() || payload["error"].is_null());
    assert!(
        payload["metrics"]["execution_ms"]
            .as_f64()
            .is_some_and(|value| value >= 0.0)
    );

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn gateway_rpc_returns_parse_error_when_body_is_not_json() {
    let app = ZhenfaGatewayBuilder::new()
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .body("{not json")
        .send()
        .await
        .unwrap_or_else(|error| panic!("rpc request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("rpc parse-error response should parse: {error}"));

    assert_eq!(payload["jsonrpc"], "2.0");
    assert!(payload["id"].is_null());
    assert_eq!(payload["error"]["code"], PARSE_ERROR_CODE);
    assert!(payload["result"].is_null());

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn gateway_rpc_returns_invalid_request_when_envelope_is_missing_method() {
    let app = ZhenfaGatewayBuilder::new()
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": "req-invalid",
            "params": {}
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("rpc request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("rpc invalid-envelope response should parse: {error}"));

    assert_eq!(payload["jsonrpc"], "2.0");
    assert_eq!(payload["id"], "req-invalid");
    assert_eq!(payload["error"]["code"], INVALID_REQUEST_CODE);
    assert!(payload["result"].is_null());

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn gateway_rpc_returns_method_not_found_error() {
    let app = ZhenfaGatewayBuilder::new()
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app).await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "unknown.method",
            "id": "req-404",
            "params": {}
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("rpc request should succeed: {error}"));
    assert!(response.status().is_success());
    let payload: serde_json::Value = response
        .json()
        .await
        .unwrap_or_else(|error| panic!("rpc response should parse: {error}"));

    assert_eq!(payload["jsonrpc"], "2.0");
    assert_eq!(payload["id"], "req-404");
    assert_eq!(payload["error"]["code"], METHOD_NOT_FOUND_CODE);
    assert!(payload.get("result").is_none() || payload["result"].is_null());

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn gateway_mounts_router_plugins() {
    let app = ZhenfaGatewayBuilder::new()
        .add_router(PingRouter)
        .build()
        .unwrap_or_else(|error| panic!("build gateway with plugin: {error}"));
    let (base_url, handle) = spawn_app(app).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{base_url}/v1/ping/echo"))
        .send()
        .await
        .unwrap_or_else(|error| panic!("ping route request should succeed: {error}"));
    assert!(response.status().is_success());
    let body = response
        .text()
        .await
        .unwrap_or_else(|error| panic!("ping route body should parse: {error}"));
    assert_eq!(body, "pong");

    let rpc_response = client
        .post(format!("{base_url}/rpc"))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "ping.echo",
            "id": "req-ping",
            "params": {}
        }))
        .send()
        .await
        .unwrap_or_else(|error| panic!("plugin rpc request should succeed: {error}"));
    let rpc_payload: serde_json::Value = rpc_response
        .json()
        .await
        .unwrap_or_else(|error| panic!("plugin rpc response should parse: {error}"));
    assert_eq!(rpc_payload["result"], "pong");

    handle.abort();
    let _ = handle.await;
}

#[test]
fn gateway_build_rejects_invalid_router_prefix() {
    let Err(error) = ZhenfaGatewayBuilder::new()
        .add_router(BadPrefixRouter)
        .build()
    else {
        panic!("invalid prefix should fail build")
    };
    assert!(
        error.to_string().contains("invalid router prefix"),
        "unexpected error: {error}"
    );
}
