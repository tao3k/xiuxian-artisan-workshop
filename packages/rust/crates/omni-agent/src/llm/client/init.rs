use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use super::super::backend::{extract_api_base_from_inference_url, parse_backend_mode};
#[cfg(feature = "agent-provider-litellm")]
use super::super::compat::litellm::LiteLlmRuntime;
use super::super::providers::{ProviderSettings, resolve_provider_settings};
use super::LlmClient;
use crate::config::load_runtime_settings;

impl LlmClient {
    pub fn new(inference_url: String, model: String, api_key: Option<String>) -> Self {
        let runtime_settings = load_runtime_settings();
        let env_backend = std::env::var("OMNI_AGENT_LLM_BACKEND")
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|raw| !raw.is_empty());
        let (backend_mode, backend_source) = if let Some(raw) = env_backend.as_deref() {
            (parse_backend_mode(Some(raw)), "env")
        } else {
            let settings_backend = runtime_settings
                .agent
                .llm_backend
                .as_deref()
                .map(str::trim)
                .map(ToString::to_string)
                .filter(|raw| !raw.is_empty());
            if let Some(raw) = settings_backend.as_deref() {
                (parse_backend_mode(Some(raw)), "settings")
            } else {
                (parse_backend_mode(None), "default")
            }
        };
        let provider_settings = resolve_provider_settings(&runtime_settings, model);
        let ProviderSettings {
            mode: litellm_provider_mode,
            source: litellm_provider_source,
            api_key_env: litellm_api_key_env,
            minimax_api_base,
            model,
            timeout_secs: inference_timeout_secs,
            max_tokens: inference_max_tokens,
            max_in_flight: inference_max_in_flight,
        } = provider_settings;
        let in_flight_gate = inference_max_in_flight.map(|limit| Arc::new(Semaphore::new(limit)));
        let inference_api_base = extract_api_base_from_inference_url(&inference_url);
        tracing::info!(
            llm_backend = backend_mode.as_str(),
            llm_backend_source = backend_source,
            litellm_provider = litellm_provider_mode.as_str(),
            litellm_provider_source = litellm_provider_source,
            litellm_api_key_env = %litellm_api_key_env,
            minimax_api_base = %minimax_api_base,
            inference_timeout_secs = inference_timeout_secs,
            inference_max_tokens = inference_max_tokens,
            inference_max_in_flight = inference_max_in_flight,
            model = %model,
            inference_api_base = %inference_api_base,
            "llm backend selected"
        );
        Self {
            client: build_http_client(),
            inference_url,
            #[cfg(feature = "agent-provider-litellm")]
            inference_api_base,
            model,
            api_key,
            backend_mode,
            backend_source,
            litellm_provider_mode,
            litellm_provider_source,
            #[cfg(feature = "agent-provider-litellm")]
            litellm_api_key_env,
            #[cfg(feature = "agent-provider-litellm")]
            minimax_api_base,
            inference_timeout_secs,
            inference_max_tokens,
            inference_max_in_flight,
            in_flight_gate,
            #[cfg(feature = "agent-provider-litellm")]
            litellm_runtime: LiteLlmRuntime::new(),
        }
    }

    /// Active backend mode label (`litellm_rs`, `http`, or `mistral_local`).
    pub fn backend_mode(&self) -> &'static str {
        self.backend_mode.as_str()
    }

    /// Backend source label (`env`, `settings`, or `default`).
    pub fn backend_source(&self) -> &'static str {
        self.backend_source
    }

    /// Active litellm provider mode (`openai` or `minimax`).
    pub fn litellm_provider_mode(&self) -> &'static str {
        self.litellm_provider_mode.as_str()
    }

    /// litellm provider source (`env`, `settings`, `default`).
    pub fn litellm_provider_source(&self) -> &'static str {
        self.litellm_provider_source
    }
}

fn build_http_client() -> reqwest::Client {
    let builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(64)
        .tcp_nodelay(true);
    match builder.build() {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to build tuned llm http client; falling back to default client"
            );
            reqwest::Client::new()
        }
    }
}
