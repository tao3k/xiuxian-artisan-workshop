//! LLM module integration harness.

mod config {
    pub(crate) use omni_agent::RuntimeSettings;
}

mod session {
    pub(crate) use omni_agent::{ChatMessage, ToolCallOut};
}

mod llm {
    #[path = "../../src/llm/backend.rs"]
    pub mod backend;
    #[path = "../../src/llm/providers/mod.rs"]
    pub mod providers;
    #[path = "../../src/llm/tools.rs"]
    pub mod tools;
    #[path = "../../src/llm/types.rs"]
    pub mod types;

    fn lint_symbol_probe() {
        let _ = std::mem::size_of::<types::ChatCompletionResponse>();
        let _ = std::mem::size_of::<types::Choice>();
        let _ = std::mem::size_of::<types::AssistantMessage>();
        let parsed = types::ChatCompletionResponse {
            choices: vec![types::Choice {
                message: types::AssistantMessage {
                    content: Some(String::new()),
                    tool_calls: None,
                },
            }],
        };
        let _ = parsed.choices.first().and_then(|choice| {
            let _ = choice.message.content.as_deref();
            choice.message.tool_calls.as_ref()
        });

        let _ = tools::PreparedTool::to_http_tool_def as fn(&tools::PreparedTool) -> types::ToolDef;
        let _ = providers::resolve_provider_settings
            as fn(&crate::config::RuntimeSettings, String) -> providers::ProviderSettings;
        let _ = providers::LiteLlmProviderMode::as_str
            as fn(providers::LiteLlmProviderMode) -> &'static str;

        #[cfg(feature = "agent-provider-litellm")]
        {
            let _ = tools::PreparedTool::to_litellm_tool;
            let _ = providers::build_minimax_provider;
            let _ = providers::build_openai_provider;
            let _ = std::mem::size_of::<providers::LiteLlmMinimaxProvider>();
            let _ = std::mem::size_of::<providers::LiteLlmOpenAIProvider>();
        }
    }

    const _: fn() = lint_symbol_probe;

    #[path = "backend.rs"]
    mod backend_tests;
    #[path = "http_request.rs"]
    mod http_request_tests;
    #[path = "provider_mode.rs"]
    mod provider_mode_tests;
}
