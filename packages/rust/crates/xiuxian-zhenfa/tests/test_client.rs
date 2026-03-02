//! Integration tests for zhenfa thin HTTP client behavior.

use serde_json::json;
use xiuxian_zhenfa::{
    INVALID_PARAMS_CODE, JsonRpcErrorObject, ZhenfaClient, ZhenfaClientError, ZhenfaGatewayBuilder,
};

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
async fn client_call_stripped_returns_result_payload() {
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
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));

    let client =
        ZhenfaClient::new(base_url).unwrap_or_else(|error| panic!("create zhenfa client: {error}"));
    let output = client
        .call_stripped(
            "wendao.search",
            json!({ "query": "agenda date:this_week" }),
            None,
        )
        .await
        .unwrap_or_else(|error| panic!("zhenfa call should succeed: {error}"));
    assert_eq!(output, "<search>agenda date:this_week</search>");

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn client_maps_jsonrpc_error_response() {
    let app = ZhenfaGatewayBuilder::new()
        .register_method_fn("wendao.search", |_params, _meta| async {
            Err(JsonRpcErrorObject::invalid_params("limit must be <= 100"))
        })
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));

    let client =
        ZhenfaClient::new(base_url).unwrap_or_else(|error| panic!("create zhenfa client: {error}"));
    let Err(error) = client
        .call(
            "wendao.search",
            json!({ "query": "agenda", "limit": 101 }),
            None,
        )
        .await
    else {
        panic!("rpc error must be surfaced");
    };

    match error {
        ZhenfaClientError::Rpc { code, .. } => {
            assert_eq!(code, INVALID_PARAMS_CODE);
        }
        other => panic!("unexpected error variant: {other}"),
    }

    handle.abort();
    let _ = handle.await;
}

#[tokio::test]
async fn client_maps_non_success_http_status() {
    let app = ZhenfaGatewayBuilder::new()
        .build()
        .unwrap_or_else(|error| panic!("build gateway: {error}"));
    let (base_url, handle) = spawn_app(app)
        .await
        .unwrap_or_else(|error| panic!("start test app: {error}"));

    let client = ZhenfaClient::new(format!("{base_url}/missing"))
        .unwrap_or_else(|error| panic!("create zhenfa client: {error}"));
    let Err(error) = client
        .call_stripped("wendao.search", json!({ "query": "agenda" }), None)
        .await
    else {
        panic!("unknown path should return non-success status");
    };

    assert!(
        matches!(error, ZhenfaClientError::HttpStatus { .. }),
        "unexpected error variant: {error}"
    );

    handle.abort();
    let _ = handle.await;
}

#[test]
fn client_rejects_non_http_base_url() {
    let Err(error) = ZhenfaClient::new("localhost:18090") else {
        panic!("non-http base URL must be rejected");
    };
    assert!(
        matches!(error, ZhenfaClientError::InvalidBaseUrl { .. }),
        "unexpected error variant: {error}",
    );
}
