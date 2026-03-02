//! HTTP gateway integration tests: validation (400), routing, response shape.
//! Uses a minimal Agent (no MCP) so no external services are required.

use axum::body::Body;
use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use omni_agent::{Agent, AgentConfig, router};
use serde_json::Value;
use tower::ServiceExt;

fn minimal_agent_config() -> AgentConfig {
    AgentConfig {
        inference_url: "https://api.openai.com/v1/chat/completions".to_string(),
        model: "gpt-4o-mini".to_string(),
        api_key: None,
        max_tool_rounds: 1,
        ..AgentConfig::default()
    }
}

async fn build_agent_or_panic() -> Agent {
    let config = minimal_agent_config();
    match Agent::from_config(config).await {
        Ok(agent) => agent,
        Err(error) => panic!("agent init should succeed: {error}"),
    }
}

fn request_or_panic(builder: axum::http::request::Builder, body: Body) -> Request<Body> {
    match builder.body(body) {
        Ok(request) => request,
        Err(error) => panic!("request build should succeed: {error}"),
    }
}

async fn oneshot_or_panic(app: axum::Router, request: Request<Body>) -> axum::response::Response {
    match app.oneshot(request).await {
        Ok(response) => response,
        Err(error) => panic!("gateway request should succeed: {error}"),
    }
}

#[tokio::test]
async fn gateway_returns_400_for_empty_session_id() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, None);

    let request = request_or_panic(
        Request::post("/message").header("content-type", "application/json"),
        Body::from(r#"{"session_id":"","message":"hi"}"#),
    );
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_returns_400_for_empty_message() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, None);

    let request = request_or_panic(
        Request::post("/message").header("content-type", "application/json"),
        Body::from(r#"{"session_id":"s1","message":"   "}"#),
    );
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_returns_404_for_unknown_route() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, None);

    let request = request_or_panic(Request::get("/unknown"), Body::empty());
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gateway_embed_returns_400_for_empty_text() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, None);

    let request = request_or_panic(
        Request::post("/embed").header("content-type", "application/json"),
        Body::from(r#"{"text":"   "}"#),
    );
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_embed_batch_returns_400_for_empty_texts() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, None);

    let request = request_or_panic(
        Request::post("/embed/batch").header("content-type", "application/json"),
        Body::from(r#"{"texts":[]}"#),
    );
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_openai_embeddings_returns_400_for_invalid_input_type() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, None);

    let request = request_or_panic(
        Request::post("/v1/embeddings").header("content-type", "application/json"),
        Body::from(r#"{"input":123}"#),
    );
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn gateway_health_returns_structured_summary_without_mcp() {
    let agent = build_agent_or_panic().await;
    let app = router(agent, 300, Some(4));

    let request = request_or_panic(Request::get("/health"), Body::empty());
    let response = oneshot_or_panic(app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let bytes_result = to_bytes(response.into_body(), usize::MAX).await;
    let bytes = match bytes_result {
        Ok(bytes) => bytes,
        Err(error) => panic!("health body bytes should decode: {error}"),
    };
    let payload: Value = match serde_json::from_slice(&bytes) {
        Ok(payload) => payload,
        Err(error) => panic!("health response should be valid JSON: {error}"),
    };

    assert_eq!(
        payload.get("status").and_then(Value::as_str),
        Some("healthy")
    );
    assert_eq!(
        payload.get("turn_timeout_secs").and_then(Value::as_u64),
        Some(300)
    );
    assert_eq!(
        payload.get("max_concurrent_turns").and_then(Value::as_u64),
        Some(4)
    );
    assert_eq!(
        payload.get("in_flight_turns").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload
            .get("mcp")
            .and_then(|mcp| mcp.get("enabled"))
            .and_then(Value::as_bool),
        Some(false)
    );
    let tools_list_cache = payload
        .get("mcp")
        .and_then(|mcp| mcp.get("tools_list_cache"));
    assert!(
        tools_list_cache.is_none() || tools_list_cache.is_some_and(Value::is_null),
        "tools_list_cache should be omitted or null when MCP is disabled"
    );
}
