//! Telegram tool-result rendering tests for markdown and fallback behavior.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use omni_agent::{
    Channel, TelegramChannel, markdown_to_telegram_html, markdown_to_telegram_markdown_v2,
};
use tokio::sync::Mutex;

#[derive(Clone, Default)]
struct MockTelegramState {
    requests: Arc<Mutex<Vec<serde_json::Value>>>,
}

async fn handle_send_message(
    State(state): State<MockTelegramState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.requests.lock().await.push(payload);
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}

async fn spawn_mock_telegram_api()
-> Result<Option<(String, MockTelegramState, tokio::task::JoinHandle<()>)>> {
    let state = MockTelegramState::default();
    let app = Router::new()
        .route("/botfake-token/sendMessage", post(handle_send_message))
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!(
                "skipping telegram tool-result render tests: local socket bind is not permitted"
            );
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;

    Ok(Some((format!("http://{addr}"), state, handle)))
}

async fn wait_for_listener(addr: std::net::SocketAddr) {
    for _ in 0..20 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

#[tokio::test]
async fn telegram_send_extracts_markdown_from_json_content_string() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let content = "# Crawl Result\n\n- item 1\n- item 2";
    let envelope = serde_json::json!({
        "success": true,
        "content": content
    });
    channel.send(&envelope.to_string(), "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(
        request
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("MarkdownV2")
    );
    let expected = markdown_to_telegram_markdown_v2(content);
    assert_eq!(
        request.get("text").and_then(serde_json::Value::as_str),
        Some(expected.as_str())
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_extracts_markdown_from_json_content_array() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let extracted_content = "# Header\nsecond line";
    let envelope = serde_json::json!({
        "success": true,
        "content": [
            {"type": "text", "text": "# Header"},
            {"type": "text", "text": "second line"}
        ]
    });
    channel.send(&envelope.to_string(), "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 1);
    let expected = markdown_to_telegram_markdown_v2(extracted_content);
    assert_eq!(
        requests[0].get("text").and_then(serde_json::Value::as_str),
        Some(expected.as_str())
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_keeps_json_payload_when_no_display_content_is_present() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let payload = serde_json::json!({
        "success": true,
        "status": "ok",
        "metrics": {
            "count": 2
        }
    })
    .to_string();

    channel.send(&payload, "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 1);
    let expected = markdown_to_telegram_markdown_v2(&payload);
    assert_eq!(
        requests[0].get("text").and_then(serde_json::Value::as_str),
        Some(expected.as_str())
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_prefers_html_for_image_markdown_blocks() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let content = "[![战争机器4](https://image.example.com/gow4.gif) 战争机器4](https://www.gamersky.com/z/gearsofwar4/)";
    let payload = serde_json::json!({
        "success": true,
        "content": content,
    })
    .to_string();

    channel.send(&payload, "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("HTML")
    );
    let expected = markdown_to_telegram_html(content);
    assert_eq!(
        requests[0].get("text").and_then(serde_json::Value::as_str),
        Some(expected.as_str())
    );

    handle.abort();
    Ok(())
}
