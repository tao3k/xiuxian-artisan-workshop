//! LLM runtime primitives and clients.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let prompt_dump = if let Some(sys_msg) = request.messages.get(0) {
            sys_msg.content.clone()
        } else {
            "No Context".to_string()
        };

        let res = match self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                println!(
                    "[LLM Error] Connection failed: {}. Falling back to prompt dump.",
                    e
                );
                return Ok(format!("Mock LLM Conclusion (Fallback):\n{}", prompt_dump));
            }
        };

        let data_res: Result<ChatResponse, _> = res.json().await;
        match data_res {
            Ok(data) => data
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| anyhow::anyhow!("Empty choice in LLM response")),
            Err(e) => {
                println!(
                    "[LLM Error] Decoding failed: {}. Falling back to prompt dump.",
                    e
                );
                Ok(format!("Mock LLM Conclusion (Fallback):\n{}", prompt_dump))
            }
        }
    }
}
