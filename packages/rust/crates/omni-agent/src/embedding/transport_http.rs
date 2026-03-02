use std::time::{Duration, Instant};

use reqwest::Client;

use super::types::EmbedBatchResponse;

const EMBED_HTTP_RETRY_BASE_DELAY_MS: u64 = 80;
const EMBED_HTTP_RETRY_MAX_DELAY_MS: u64 = 1_000;
const EMBED_HTTP_MAX_ATTEMPTS: usize = 8;

pub(crate) async fn embed_http(
    client: &Client,
    base_url: &str,
    texts: &[String],
    model: Option<&str>,
) -> Option<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Some(vec![]);
    }
    let started = Instant::now();
    let url = format!("{base_url}/embed/batch");
    let mut body = serde_json::json!({ "texts": texts });
    if let Some(model) = model.map(str::trim).filter(|value| !value.is_empty()) {
        body["model"] = serde_json::Value::String(model.to_string());
    }
    for attempt in 1..=EMBED_HTTP_MAX_ATTEMPTS {
        let resp = match client.post(&url).json(&body).send().await {
            Ok(resp) => resp,
            Err(error) => {
                let should_retry =
                    attempt < EMBED_HTTP_MAX_ATTEMPTS && should_retry_http_request_error(&error);
                if should_retry {
                    let retry_delay = retry_delay_for_attempt(attempt);
                    tracing::debug!(
                        event = "agent.embedding.http.request_retry",
                        url,
                        attempt,
                        max_attempts = EMBED_HTTP_MAX_ATTEMPTS,
                        retry_delay_ms = retry_delay.as_millis(),
                        elapsed_ms = started.elapsed().as_millis(),
                        error = %error,
                        "embedding http request failed; retrying"
                    );
                    tokio::time::sleep(retry_delay).await;
                    continue;
                }
                tracing::debug!(
                    event = "agent.embedding.http.request_failed",
                    url,
                    attempt,
                    max_attempts = EMBED_HTTP_MAX_ATTEMPTS,
                    elapsed_ms = started.elapsed().as_millis(),
                    error = %error,
                    "embedding http request failed"
                );
                return None;
            }
        };
        if !resp.status().is_success() {
            let should_retry = attempt < EMBED_HTTP_MAX_ATTEMPTS && resp.status().is_server_error();
            if should_retry {
                let retry_delay = retry_delay_for_attempt(attempt);
                tracing::debug!(
                    event = "agent.embedding.http.retry_on_server_error",
                    status = %resp.status(),
                    attempt,
                    max_attempts = EMBED_HTTP_MAX_ATTEMPTS,
                    retry_delay_ms = retry_delay.as_millis(),
                    elapsed_ms = started.elapsed().as_millis(),
                    "embedding http returned server error; retrying"
                );
                tokio::time::sleep(retry_delay).await;
                continue;
            }
            tracing::debug!(
                event = "agent.embedding.http.non_success_status",
                status = %resp.status(),
                attempt,
                max_attempts = EMBED_HTTP_MAX_ATTEMPTS,
                elapsed_ms = started.elapsed().as_millis(),
                "embedding http returned non-success status"
            );
            return None;
        }
        let data: EmbedBatchResponse = match resp.json().await {
            Ok(data) => data,
            Err(error) => {
                tracing::debug!(
                    event = "agent.embedding.http.decode_failed",
                    elapsed_ms = started.elapsed().as_millis(),
                    attempt,
                    max_attempts = EMBED_HTTP_MAX_ATTEMPTS,
                    error = %error,
                    "embedding http response decode failed"
                );
                return None;
            }
        };
        let vectors = data.vectors;
        tracing::debug!(
            event = "agent.embedding.http.completed",
            elapsed_ms = started.elapsed().as_millis(),
            attempt,
            max_attempts = EMBED_HTTP_MAX_ATTEMPTS,
            success = vectors.is_some(),
            "embedding http path completed"
        );
        return vectors;
    }
    None
}

fn should_retry_http_request_error(error: &reqwest::Error) -> bool {
    error.is_connect()
        || error.is_timeout()
        || error.to_string().contains("error sending request for url")
}

fn retry_delay_for_attempt(attempt: usize) -> Duration {
    let attempt_index = attempt.saturating_sub(1);
    let shift = u32::try_from(attempt_index.min(6)).unwrap_or(6);
    let factor = 1_u64.checked_shl(shift).unwrap_or(u64::MAX);
    let delay_ms = EMBED_HTTP_RETRY_BASE_DELAY_MS.saturating_mul(factor);
    Duration::from_millis(delay_ms.min(EMBED_HTTP_RETRY_MAX_DELAY_MS))
}
