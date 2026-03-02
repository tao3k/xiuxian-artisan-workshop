//! LLM runtime primitives and clients.

use async_trait::async_trait;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use tracing::error;

/// Represents a single message in a chat conversation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    /// Message role, e.g. `system`, `user`, `assistant`, or `tool`.
    pub role: String,
    /// Message text content.
    pub content: String,
}

/// A request to the LLM for chat completion.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRequest {
    /// Target model identifier.
    pub model: String,
    /// Ordered chat messages sent to the model.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature.
    pub temperature: f32,
}

/// A single choice returned by the LLM.
#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    /// Message payload for this completion choice.
    pub message: ChatMessage,
}

/// The envelope response from the LLM.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// Candidate completions returned by the model.
    pub choices: Vec<ChatChoice>,
}

/// The core trait for interacting with Large Language Models.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Execute a chat-completion request and return the first text answer.
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<String>;
}

/// Standard OpenAI-compatible HTTP client.
pub struct OpenAIClient {
    /// API key used for bearer-token authorization.
    pub api_key: String,
    /// Base URL for the OpenAI-compatible endpoint.
    pub base_url: String,
    /// Shared HTTP client.
    pub http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorEnvelope {
    error: OpenAiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorDetail {
    message: Option<String>,
}

fn body_preview(body: &str) -> String {
    const PREVIEW_LIMIT: usize = 320;
    let compact = body.replace(['\n', '\r'], " ").trim().to_string();
    if compact.len() <= PREVIEW_LIMIT {
        compact
    } else {
        format!("{}...", &compact[..PREVIEW_LIMIT])
    }
}

fn extract_openai_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<OpenAiErrorEnvelope>(body)
        .ok()
        .and_then(|payload| payload.error.message)
        .map(|message| message.trim().to_string())
        .filter(|message| !message.is_empty())
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let res = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!("LLM Connection failed: {}", e);
                anyhow::anyhow!("LLM Connection failed: {e}")
            })?;

        let status = res.status();
        let content_type = res
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|header| header.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let body = res.text().await.map_err(|e| {
            error!("LLM response body read failed: {}", e);
            anyhow::anyhow!("LLM response body read failed: {e}")
        })?;

        if !status.is_success() {
            let provider_message = extract_openai_error_message(&body);
            let preview = body_preview(&body);
            let reason = provider_message.unwrap_or_else(|| preview.clone());
            error!(
                "LLM request failed: status={}, content_type={}, reason={}",
                status, content_type, reason
            );
            return Err(anyhow::anyhow!(
                "LLM request failed with status {status} (content-type: {content_type}): {reason}"
            ));
        }

        let data_res: Result<ChatResponse, _> = serde_json::from_str(&body);
        let data = data_res.map_err(|e| {
            let provider_message = extract_openai_error_message(&body);
            let preview = body_preview(&body);
            let reason = provider_message.unwrap_or(preview);
            error!(
                "LLM Response Decoding failed: {} (status={}, content_type={}, body_preview={})",
                e, status, content_type, reason
            );
            anyhow::anyhow!(
                "LLM Response Decoding failed: {e} (status={status}, content-type={content_type}, body_preview={reason})"
            )
        })?;

        data.choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("Empty choice in LLM response"))
    }
}
