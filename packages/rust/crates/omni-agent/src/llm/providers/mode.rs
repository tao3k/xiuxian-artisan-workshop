use crate::config::RuntimeSettings;
use xiuxian_macros::{env_non_empty, string_first_non_empty};

use super::minimax::{normalize_minimax_api_base, normalize_minimax_model};
use super::{DEFAULT_MINIMAX_KEY_ENV, DEFAULT_OPENAI_KEY_ENV};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::llm) enum LiteLlmProviderMode {
    OpenAi,
    Minimax,
}

impl LiteLlmProviderMode {
    pub(in crate::llm) fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Minimax => "minimax",
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::llm) struct ProviderSettings {
    pub mode: LiteLlmProviderMode,
    pub source: &'static str,
    pub api_key_env: String,
    pub minimax_api_base: String,
    pub model: String,
    pub timeout_secs: u64,
    pub max_tokens: Option<u32>,
    pub max_in_flight: Option<usize>,
}

fn parse_litellm_provider_mode(raw: Option<&str>) -> LiteLlmProviderMode {
    match raw.map(str::trim).map(str::to_ascii_lowercase) {
        Some(value) if value == "minimax" => LiteLlmProviderMode::Minimax,
        Some(value) if value.is_empty() => LiteLlmProviderMode::OpenAi,
        None => LiteLlmProviderMode::OpenAi,
        Some(value) => {
            tracing::warn!(
                provider = %value,
                "unsupported litellm provider; using openai provider mode"
            );
            LiteLlmProviderMode::OpenAi
        }
    }
}

pub(in crate::llm) fn resolve_provider_settings(
    runtime_settings: &RuntimeSettings,
    requested_model: String,
) -> ProviderSettings {
    let env_provider = env_non_empty!("OMNI_AGENT_LLM_PROVIDER");
    let env_minimax_api_base = env_non_empty!("MINIMAX_API_BASE");
    resolve_provider_settings_with_env(
        runtime_settings,
        requested_model,
        env_provider.as_deref(),
        env_minimax_api_base.as_deref(),
    )
}

pub(in crate::llm) fn resolve_provider_settings_with_env(
    runtime_settings: &RuntimeSettings,
    requested_model: String,
    env_provider_raw: Option<&str>,
    env_minimax_api_base_raw: Option<&str>,
) -> ProviderSettings {
    let env_provider = env_provider_raw
        .map(str::trim)
        .filter(|raw| !raw.is_empty());
    let (mode, source) = if let Some(raw) = env_provider {
        (parse_litellm_provider_mode(Some(raw)), "env")
    } else {
        let settings_provider = runtime_settings
            .inference
            .provider
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(raw) = settings_provider {
            (parse_litellm_provider_mode(Some(raw)), "settings")
        } else {
            (LiteLlmProviderMode::OpenAi, "default")
        }
    };

    let api_key_env = string_first_non_empty!(
        runtime_settings.inference.api_key_env.as_deref(),
        match mode {
            LiteLlmProviderMode::OpenAi => Some(DEFAULT_OPENAI_KEY_ENV),
            LiteLlmProviderMode::Minimax => Some(DEFAULT_MINIMAX_KEY_ENV),
        },
    );

    let settings_model = runtime_settings
        .inference
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let model = if requested_model.trim().is_empty() {
        settings_model.unwrap_or_else(|| match mode {
            LiteLlmProviderMode::Minimax => "MiniMax-M2.5".to_string(),
            LiteLlmProviderMode::OpenAi => requested_model,
        })
    } else {
        requested_model
    };
    let model = match mode {
        LiteLlmProviderMode::Minimax => normalize_minimax_model(&model),
        LiteLlmProviderMode::OpenAi => model,
    };

    let minimax_api_base = normalize_minimax_api_base(
        env_minimax_api_base_raw
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or(runtime_settings.inference.base_url.as_deref()),
    );
    let timeout_secs = runtime_settings
        .inference
        .timeout
        .filter(|value| *value > 0)
        .unwrap_or(60);
    let max_tokens = runtime_settings
        .inference
        .max_tokens
        .filter(|value| *value > 0)
        .map(|value| u32::try_from(value.min(u64::from(u32::MAX))).unwrap_or(u32::MAX));
    let max_in_flight = runtime_settings
        .inference
        .max_in_flight
        .filter(|value| *value > 0);

    ProviderSettings {
        mode,
        source,
        api_key_env,
        minimax_api_base,
        model,
        timeout_secs,
        max_tokens,
        max_in_flight,
    }
}
