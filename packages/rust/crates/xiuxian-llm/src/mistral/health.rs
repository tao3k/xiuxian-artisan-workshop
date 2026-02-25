//! Mistral OpenAI-compatible readiness probes.

use std::time::Duration;

/// Health probe result for Mistral OpenAI-compatible runtime.
#[derive(Debug, Clone)]
pub struct MistralHealthStatus {
    /// Probe succeeded and endpoint returned 2xx.
    pub ready: bool,
    /// HTTP status code, when available.
    pub status_code: Option<u16>,
    /// Request hit timeout.
    pub timed_out: bool,
    /// Request failed due to transport/network.
    pub transport_error: bool,
    /// Compact summary suitable for structured logs.
    pub summary: String,
}

/// Normalize base URL to a concrete OpenAI-compatible `/v1/models` URL.
#[must_use]
pub fn derive_models_url(base_url: &str) -> Option<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.ends_with("/v1/models") {
        return Some(trimmed.to_string());
    }
    if trimmed.ends_with("/v1") {
        return Some(format!("{trimmed}/models"));
    }
    Some(format!("{trimmed}/v1/models"))
}

/// Probe `/v1/models` readiness on a Mistral OpenAI-compatible server.
pub async fn probe_models(base_url: &str, timeout_ms: u64) -> MistralHealthStatus {
    let Some(models_url) = derive_models_url(base_url) else {
        return MistralHealthStatus {
            ready: false,
            status_code: None,
            timed_out: false,
            transport_error: false,
            summary: "mistral_health_probe_skipped(invalid_base_url)".to_string(),
        };
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(1)))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return MistralHealthStatus {
                ready: false,
                status_code: None,
                timed_out: false,
                transport_error: false,
                summary: format!("mistral_health_probe_build_failed({error})"),
            };
        }
    };

    match client.get(&models_url).send().await {
        Ok(response) => {
            let status = response.status();
            MistralHealthStatus {
                ready: status.is_success(),
                status_code: Some(status.as_u16()),
                timed_out: false,
                transport_error: false,
                summary: format!("mistral_health_status={}", status.as_u16()),
            }
        }
        Err(error) => {
            if error.is_timeout() {
                return MistralHealthStatus {
                    ready: false,
                    status_code: None,
                    timed_out: true,
                    transport_error: false,
                    summary: "mistral_health_timeout".to_string(),
                };
            }
            MistralHealthStatus {
                ready: false,
                status_code: None,
                timed_out: false,
                transport_error: true,
                summary: format!("mistral_health_error({error})"),
            }
        }
    }
}
