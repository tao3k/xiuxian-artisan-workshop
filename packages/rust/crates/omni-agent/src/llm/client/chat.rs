use std::time::{Duration, Instant};

use anyhow::Result;

use super::super::backend::LlmBackendMode;
#[cfg(feature = "agent-provider-litellm")]
use super::super::compat::litellm::LiteLlmDispatchConfig;
use super::super::tools::{PreparedTool, parse_tools_json};
use super::super::types::{AssistantMessage, ChatCompletionRequest, ChatCompletionResponse};
use super::LlmClient;
use crate::session::ChatMessage;

impl LlmClient {
    /// Send messages and optionally tool definitions; returns content and/or `tool_calls`.
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools_json: Option<Vec<serde_json::Value>>,
    ) -> Result<AssistantMessage> {
        let tools = parse_tools_json(tools_json);
        let started_at = Instant::now();
        let gate_wait_started = Instant::now();
        let _in_flight_permit = if let Some(gate) = self.in_flight_gate.as_ref() {
            Some(
                gate.clone()
                    .acquire_owned()
                    .await
                    .map_err(|_| anyhow::anyhow!("llm in-flight gate closed unexpectedly"))?,
            )
        } else {
            None
        };
        let gate_wait_ms =
            u64::try_from(gate_wait_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::debug!(
            event = "agent.llm.chat.dispatch",
            llm_backend = self.backend_mode(),
            llm_backend_source = self.backend_source(),
            litellm_provider = self.litellm_provider_mode(),
            litellm_provider_source = self.litellm_provider_source(),
            inference_max_in_flight = self.inference_max_in_flight,
            gate_wait_ms = gate_wait_ms,
            message_count = messages.len(),
            tools_count = tools.len(),
            "dispatching llm chat request"
        );
        let result = match self.backend_mode {
            LlmBackendMode::OpenAiCompatibleHttp | LlmBackendMode::MistralLocal => {
                self.chat_via_http(messages, tools).await
            }
            LlmBackendMode::LiteLlmRs => {
                #[cfg(feature = "agent-provider-litellm")]
                {
                    self.chat_via_litellm_rs(messages, tools).await
                }
                #[cfg(not(feature = "agent-provider-litellm"))]
                {
                    let _ = (messages, tools);
                    Err(anyhow::anyhow!(
                        "litellm-rs backend is disabled at compile time (feature agent-provider-litellm)"
                    ))
                }
            }
        };
        let elapsed_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        match &result {
            Ok(message) => {
                let tool_call_count = message.tool_calls.as_ref().map_or(0, std::vec::Vec::len);
                tracing::debug!(
                    event = "agent.llm.chat.completed",
                    llm_backend = self.backend_mode(),
                    litellm_provider = self.litellm_provider_mode(),
                    elapsed_ms = elapsed_ms,
                    tool_call_count = tool_call_count,
                    "llm chat request completed"
                );
            }
            Err(error) => {
                tracing::warn!(
                    event = "agent.llm.chat.failed",
                    llm_backend = self.backend_mode(),
                    litellm_provider = self.litellm_provider_mode(),
                    elapsed_ms = elapsed_ms,
                    error = %error,
                    "llm chat request failed"
                );
            }
        }
        result
    }

    async fn chat_via_http(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<PreparedTool>,
    ) -> Result<AssistantMessage> {
        let tools = if tools.is_empty() {
            None
        } else {
            Some(tools.iter().map(PreparedTool::to_http_tool_def).collect())
        };
        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            tool_choice: tools.as_ref().map(|_| "auto".to_string()),
            tools,
            max_tokens: self.inference_max_tokens,
        };
        let mut request = self
            .client
            .post(&self.inference_url)
            .json(&body)
            .header("Content-Type", "application/json");
        request = request.timeout(Duration::from_secs(self.inference_timeout_secs));
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {key}"));
        }
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!("LLM API error {status}: {text}"));
        }
        let parsed: ChatCompletionResponse = serde_json::from_str(&text)
            .map_err(|error| anyhow::anyhow!("LLM response parse error: {error}; body: {text}"))?;
        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("LLM response has no choices"))?;
        Ok(choice.message)
    }

    #[cfg(feature = "agent-provider-litellm")]
    async fn chat_via_litellm_rs(
        &self,
        messages: Vec<ChatMessage>,
        tools: Vec<PreparedTool>,
    ) -> Result<AssistantMessage> {
        self.litellm_runtime
            .chat(
                LiteLlmDispatchConfig {
                    provider_mode: self.litellm_provider_mode,
                    model: &self.model,
                    max_tokens: self.inference_max_tokens,
                    api_key: self.api_key.as_deref(),
                    litellm_api_key_env: &self.litellm_api_key_env,
                    inference_api_base: &self.inference_api_base,
                    minimax_api_base: &self.minimax_api_base,
                    timeout_secs: self.inference_timeout_secs,
                },
                messages,
                tools,
            )
            .await
    }
}
