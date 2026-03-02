use std::time::Duration;

use xiuxian_macros::{env_first_non_empty, env_non_empty};

#[cfg(feature = "agent-provider-litellm")]
use crate::config::load_runtime_settings;

pub(super) fn build_chunk_ranges(total: usize, max_chunk_size: usize) -> Vec<(usize, usize)> {
    if total == 0 {
        return Vec::new();
    }
    let chunk = max_chunk_size.max(1);
    let mut ranges = Vec::with_capacity(total.div_ceil(chunk));
    let mut start = 0usize;
    while start < total {
        let end = (start + chunk).min(total);
        ranges.push((start, end));
        start = end;
    }
    ranges
}

pub(super) fn build_http_client(timeout_secs: u64) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(64)
        .tcp_nodelay(true);
    if !system_proxy_enabled() {
        builder = builder.no_proxy();
    }
    match builder.build() {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!(
                error = %error,
                "failed to build tuned embedding http client; falling back to default client"
            );
            reqwest::Client::new()
        }
    }
}

fn system_proxy_enabled() -> bool {
    env_non_empty!("OMNI_AGENT_HTTP_ENABLE_SYSTEM_PROXY")
        .map(|raw| raw.trim().to_ascii_lowercase())
        .is_some_and(|raw| matches!(raw.as_str(), "1" | "true" | "yes" | "on"))
}

#[cfg(feature = "agent-provider-litellm")]
pub(super) struct LitellmEmbedApiKeyResolution {
    pub(super) api_key: Option<String>,
    pub(super) source: String,
}

#[cfg(feature = "agent-provider-litellm")]
pub(super) fn resolve_litellm_embed_api_key() -> LitellmEmbedApiKeyResolution {
    let read_from_env = |name: &str| env_non_empty!(name);
    let from_named_env = |name: &str| {
        read_from_env(name).map(|value| LitellmEmbedApiKeyResolution {
            api_key: Some(value),
            source: name.to_string(),
        })
    };

    if let Some(result) = from_named_env("OMNI_AGENT_EMBED_API_KEY") {
        return result;
    }
    if let Some(result) = from_named_env("LITELLM_API_KEY") {
        return result;
    }

    let runtime_settings = load_runtime_settings();
    if let Some(configured_env_name) = runtime_settings
        .inference
        .api_key_env
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
        && let Some(value) = read_from_env(&configured_env_name)
    {
        return LitellmEmbedApiKeyResolution {
            api_key: Some(value),
            source: configured_env_name,
        };
    }

    if let Some(api_key) = env_first_non_empty!("MINIMAX_API_KEY", "OPENAI_API_KEY") {
        let source = if env_non_empty!("MINIMAX_API_KEY").is_some() {
            "MINIMAX_API_KEY"
        } else {
            "OPENAI_API_KEY"
        };
        return LitellmEmbedApiKeyResolution {
            api_key: Some(api_key),
            source: source.to_string(),
        };
    }

    LitellmEmbedApiKeyResolution {
        api_key: None,
        source: "none".to_string(),
    }
}

pub(super) fn parse_positive_env_u64(name: &str, default_value: u64, max_value: u64) -> u64 {
    let value = env_non_empty!(name)
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value);
    value.min(max_value)
}

pub(super) fn parse_positive_env_usize(
    name: &str,
    default_value: usize,
    max_value: usize,
) -> usize {
    let value = env_non_empty!(name)
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value);
    value.min(max_value)
}
