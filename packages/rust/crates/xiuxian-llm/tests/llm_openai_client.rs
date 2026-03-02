//! Integration tests for OpenAI-compatible chat client behavior.

use anyhow::{Result, anyhow};
use axum::extract::State;
use axum::http::{StatusCode, header::CONTENT_TYPE};
use axum::routing::post;
use axum::{Router, response::IntoResponse};
use tokio::net::TcpListener;
use xiuxian_llm::llm::{ChatMessage, ChatRequest, LlmClient, OpenAIClient};

#[derive(Clone)]
struct MockResponse {
    status: StatusCode,
    content_type: &'static str,
    body: &'static str,
}

async fn chat_completions(State(state): State<MockResponse>) -> impl IntoResponse {
    (
        state.status,
        [(CONTENT_TYPE, state.content_type)],
        state.body.to_string(),
    )
}

async fn spawn_mock_openai_server(state: MockResponse) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state);
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(format!("http://{addr}/v1"))
}

fn request() -> ChatRequest {
    ChatRequest {
        model: "test-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        }],
        temperature: 0.1,
    }
}

#[tokio::test]
async fn openai_client_chat_success_returns_first_choice_content() -> Result<()> {
    let base_url = spawn_mock_openai_server(MockResponse {
        status: StatusCode::OK,
        content_type: "application/json",
        body: r#"{
          "choices": [
            {
              "message": {
                "role": "assistant",
                "content": "ok"
              }
            }
          ]
        }"#,
    })
    .await?;

    let client = OpenAIClient {
        api_key: "test".to_string(),
        base_url,
        http: reqwest::Client::new(),
    };

    let result = client.chat(request()).await?;
    assert_eq!(result, "ok");
    Ok(())
}

#[tokio::test]
async fn openai_client_chat_non_success_status_surfaces_provider_message() -> Result<()> {
    let base_url = spawn_mock_openai_server(MockResponse {
        status: StatusCode::BAD_REQUEST,
        content_type: "application/json",
        body: r#"{"error":{"message":"model not found"}}"#,
    })
    .await?;

    let client = OpenAIClient {
        api_key: "test".to_string(),
        base_url,
        http: reqwest::Client::new(),
    };

    let Err(err) = client.chat(request()).await else {
        return Err(anyhow!("chat should fail"));
    };
    let text = err.to_string();
    assert!(text.contains("status 400"), "unexpected error: {text}");
    assert!(text.contains("model not found"), "unexpected error: {text}");
    Ok(())
}

#[tokio::test]
async fn openai_client_chat_decode_error_includes_body_preview() -> Result<()> {
    let base_url = spawn_mock_openai_server(MockResponse {
        status: StatusCode::OK,
        content_type: "text/plain",
        body: "upstream unavailable",
    })
    .await?;

    let client = OpenAIClient {
        api_key: "test".to_string(),
        base_url,
        http: reqwest::Client::new(),
    };

    let Err(err) = client.chat(request()).await else {
        return Err(anyhow!("chat should fail"));
    };
    let text = err.to_string();
    assert!(
        text.contains("LLM Response Decoding failed"),
        "unexpected error: {text}"
    );
    assert!(
        text.contains("upstream unavailable"),
        "unexpected error: {text}"
    );
    Ok(())
}
