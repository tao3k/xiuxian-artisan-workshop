use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use xiuxian_llm::mistral::ManagedMistralServer;

use crate::agent::Agent;
use crate::embedding::EmbeddingClient;
use crate::mcp::McpToolsListCacheStatsSnapshot;

/// Request body for POST /message.
#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    /// Conversation session identifier.
    pub session_id: String,
    /// User message to send to the agent.
    pub message: String,
}

/// Response body.
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    /// Agent reply (text output).
    pub output: String,
    /// Session identifier (echo of request).
    pub session_id: String,
}

/// Shared state for the HTTP server: agent + per-turn timeout + optional concurrency limit.
#[derive(Clone)]
pub struct GatewayState {
    pub agent: Arc<Agent>,
    pub turn_timeout_secs: u64,
    /// When Some, limits concurrent agent turns; excess requests wait for a slot.
    pub concurrency_semaphore: Option<Arc<Semaphore>>,
    pub max_concurrent_turns: Option<usize>,
    pub embedding_runtime: Arc<GatewayEmbeddingRuntime>,
}

#[derive(Clone)]
pub struct GatewayEmbeddingRuntime {
    pub client: Arc<EmbeddingClient>,
    pub default_model: Option<String>,
    pub managed_mistral_server: Option<Arc<Mutex<ManagedMistralServer>>>,
}

#[derive(Debug, Deserialize)]
pub struct EmbedBatchRequest {
    pub texts: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbedBatchResponse {
    pub vectors: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub text: String,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmbedResponse {
    pub vector: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiEmbeddingsRequest {
    pub input: serde_json::Value,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenAiEmbeddingsResponse {
    pub object: &'static str,
    pub model: String,
    pub data: Vec<OpenAiEmbeddingData>,
    pub usage: OpenAiEmbeddingUsage,
}

#[derive(Debug, Serialize)]
pub struct OpenAiEmbeddingData {
    pub object: &'static str,
    pub index: usize,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct OpenAiEmbeddingUsage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

/// MCP section in gateway health response.
#[derive(Debug, Serialize)]
pub struct GatewayMcpHealthResponse {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_list_cache: Option<McpToolsListCacheStatsSnapshot>,
}

/// Response body for gateway health endpoint.
#[derive(Debug, Serialize)]
pub struct GatewayHealthResponse {
    pub status: &'static str,
    pub turn_timeout_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_turns: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_flight_turns: Option<usize>,
    pub mcp: GatewayMcpHealthResponse,
}

pub type GatewayJsonError = (axum::http::StatusCode, String);
pub type GatewayJsonResult<T> = Result<Json<T>, GatewayJsonError>;
