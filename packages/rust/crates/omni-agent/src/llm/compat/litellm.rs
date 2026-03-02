use anyhow::Result;
use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use litellm_rs::core::types::{
    ChatRequest as LiteChatRequest, ChatResponse as LiteChatResponse,
    RequestContext as LiteRequestContext, ToolChoice as LiteToolChoice,
};
use tokio::sync::OnceCell;
use xiuxian_macros::env_first_non_empty;

use crate::llm::converters::{
    chat_message_to_litellm_message, content_from_litellm, tool_call_from_litellm,
};
use crate::llm::providers::{
    DEFAULT_MINIMAX_KEY_ENV, DEFAULT_OPENAI_KEY_ENV, LiteLlmMinimaxProvider, LiteLlmOpenAIProvider,
    LiteLlmProviderMode, build_minimax_provider, build_openai_provider,
};
use crate::llm::tools::PreparedTool;
use crate::llm::types::AssistantMessage;
use crate::session::ChatMessage;

/// Dispatch settings for a single `litellm-rs` chat request.
pub(in crate::llm) struct LiteLlmDispatchConfig<'a> {
    pub(in crate::llm) provider_mode: LiteLlmProviderMode,
    pub(in crate::llm) model: &'a str,
    pub(in crate::llm) max_tokens: Option<u32>,
    pub(in crate::llm) api_key: Option<&'a str>,
    pub(in crate::llm) litellm_api_key_env: &'a str,
    pub(in crate::llm) inference_api_base: &'a str,
    pub(in crate::llm) minimax_api_base: &'a str,
    pub(in crate::llm) timeout_secs: u64,
}

/// Runtime compatibility adapter that isolates `litellm-rs` provider lifecycle.
pub(in crate::llm) struct LiteLlmRuntime {
    openai_provider: OnceCell<LiteLlmOpenAIProvider>,
    minimax_provider: OnceCell<LiteLlmMinimaxProvider>,
}

impl LiteLlmRuntime {
    #[must_use]
    pub(in crate::llm) fn new() -> Self {
        Self {
            openai_provider: OnceCell::const_new(),
            minimax_provider: OnceCell::const_new(),
        }
    }

    pub(in crate::llm) async fn chat(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        messages: Vec<ChatMessage>,
        tools: Vec<PreparedTool>,
    ) -> Result<AssistantMessage> {
        let request = Self::build_request(config.model, config.max_tokens, messages, &tools)?;
        match config.provider_mode {
            LiteLlmProviderMode::OpenAi => self.chat_openai(config, request).await,
            LiteLlmProviderMode::Minimax => self.chat_minimax(config, request).await,
        }
    }

    fn build_request(
        model: &str,
        max_tokens: Option<u32>,
        messages: Vec<ChatMessage>,
        tools: &[PreparedTool],
    ) -> Result<LiteChatRequest> {
        let tools = if tools.is_empty() {
            None
        } else {
            Some(tools.iter().map(PreparedTool::to_litellm_tool).collect())
        };
        Ok(LiteChatRequest {
            model: model.to_string(),
            messages: messages
                .into_iter()
                .map(chat_message_to_litellm_message)
                .collect::<Result<Vec<_>>>()?,
            tools: tools.clone(),
            tool_choice: tools
                .as_ref()
                .map(|_| LiteToolChoice::String("auto".to_string())),
            max_tokens,
            ..Default::default()
        })
    }

    async fn chat_openai(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        let provider = self
            .openai_provider
            .get_or_try_init(|| async {
                let api_key = resolve_litellm_api_key(
                    config.api_key,
                    config.litellm_api_key_env,
                    DEFAULT_OPENAI_KEY_ENV,
                );
                build_openai_provider(
                    config.inference_api_base.to_string(),
                    api_key,
                    config.timeout_secs,
                )
                .await
            })
            .await?;

        let response = LLMProvider::chat_completion(provider, request, LiteRequestContext::new())
            .await
            .map_err(|e| anyhow::anyhow!("litellm-rs chat completion failed: {e}"))?;
        chat_response_to_assistant(response)
    }

    async fn chat_minimax(
        &self,
        config: LiteLlmDispatchConfig<'_>,
        request: LiteChatRequest,
    ) -> Result<AssistantMessage> {
        let provider = self
            .minimax_provider
            .get_or_try_init(|| async {
                let api_key = resolve_litellm_api_key(
                    config.api_key,
                    config.litellm_api_key_env,
                    DEFAULT_MINIMAX_KEY_ENV,
                )
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "missing minimax api key; set {} or {}",
                        config.litellm_api_key_env,
                        DEFAULT_MINIMAX_KEY_ENV
                    )
                })?;
                build_minimax_provider(
                    config.minimax_api_base.to_string(),
                    api_key,
                    config.timeout_secs,
                )
                .await
            })
            .await?;

        let response = LLMProvider::chat_completion(provider, request, LiteRequestContext::new())
            .await
            .map_err(|e| anyhow::anyhow!("litellm-rs minimax chat completion failed: {e}"))?;
        chat_response_to_assistant(response)
    }
}

fn resolve_litellm_api_key(
    explicit_api_key: Option<&str>,
    primary_env: &str,
    fallback_env: &str,
) -> Option<String> {
    explicit_api_key
        .map(str::to_string)
        .or_else(|| env_first_non_empty!(primary_env, fallback_env))
}

fn chat_response_to_assistant(response: LiteChatResponse) -> Result<AssistantMessage> {
    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("litellm-rs response has no choices"))?;
    Ok(AssistantMessage {
        content: content_from_litellm(choice.message.content),
        tool_calls: choice
            .message
            .tool_calls
            .map(|calls| calls.into_iter().map(tool_call_from_litellm).collect()),
    })
}
