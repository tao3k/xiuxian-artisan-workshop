//! Top-level integration harness for `gateway::http::llm_proxy`.

mod config {
    use std::collections::HashMap;

    pub(crate) use omni_agent::load_runtime_settings;

    #[derive(Debug, Clone, Default)]
    pub(crate) struct XiuxianConfig {
        pub(crate) llm: LlmConfig,
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct LlmConfig {
        pub(crate) default_provider: Option<String>,
        pub(crate) default_model: Option<String>,
        pub(crate) providers: HashMap<String, LlmProviderConfig>,
    }

    #[derive(Debug, Clone, Default)]
    pub(crate) struct LlmProviderConfig {
        pub(crate) base_url: Option<String>,
        pub(crate) api_key_env: Option<String>,
        pub(crate) model_aliases: HashMap<String, String>,
    }

    pub(crate) fn load_xiuxian_config() -> XiuxianConfig {
        XiuxianConfig::default()
    }
}

mod gateway {
    pub(crate) mod http {
        pub(crate) mod llm_proxy {
            include!("../src/gateway/http/llm_proxy.rs");

            fn lint_symbol_probe() {
                let _ = handle_chat_completions;
                let _ = resolve_target_base_url as fn(Option<&str>, &str) -> String;
                let _ = resolve_target_api_key_env as fn(Option<&str>, &str) -> String;
                let _ = read_api_key as fn(&str) -> String;
                let _ = resolve_request_model
                    as fn(Option<&str>, Option<&str>, Option<&str>) -> Option<String>;
            }

            const _: fn() = lint_symbol_probe;

            mod tests {
                include!("gateway/http/llm_proxy.rs");
            }
        }
    }
}
