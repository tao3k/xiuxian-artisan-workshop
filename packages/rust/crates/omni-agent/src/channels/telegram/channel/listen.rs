use std::time::Duration;

use reqwest::StatusCode;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::channels::traits::ChannelMessage;

use super::TelegramChannel;
use super::constants::{
    TELEGRAM_POLL_CONFLICT_RETRY_SECS, TELEGRAM_POLL_DEFAULT_RATE_LIMIT_RETRY_SECS,
    TELEGRAM_POLL_MAX_RATE_LIMIT_RETRY_SECS, TELEGRAM_POLL_RETRY_SECS,
};
use super::error::telegram_api_error_retry_after_secs;

enum PollOutcome {
    RetryAfterSeconds(u64),
    Updates(Vec<Value>),
}

impl TelegramChannel {
    pub(super) async fn listen_updates(
        &self,
        tx: mpsc::Sender<ChannelMessage>,
    ) -> anyhow::Result<()> {
        let mut offset: i64 = 0;
        tracing::info!("Telegram channel listening for messages...");

        // NATIVE: Sync commands to Bot Menu on startup.
        // We do this before the loop to ensure they are set.
        if let Err(e) = self.sync_bot_commands().await {
            tracing::warn!("Bot command sync failed: {e}");
        }

        loop {
            match self.poll_updates(offset).await? {
                PollOutcome::RetryAfterSeconds(delay_secs) => {
                    tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                }
                PollOutcome::Updates(updates) => {
                    let should_continue = self
                        .process_polled_updates(&tx, &updates, &mut offset)
                        .await;
                    if !should_continue {
                        return Ok(());
                    }
                }
            }
        }
    }

    async fn poll_updates(&self, offset: i64) -> anyhow::Result<PollOutcome> {
        let url = self.api_url("getUpdates");
        let body = serde_json::json!({
            "offset": offset,
            "timeout": 30,
            "allowed_updates": ["message"]
        });
        let resp = match self.client.post(&url).json(&body).send().await {
            Ok(resp) => resp,
            Err(error) => {
                tracing::warn!("Telegram poll error: {error}");
                return Ok(PollOutcome::RetryAfterSeconds(TELEGRAM_POLL_RETRY_SECS));
            }
        };
        let http_status = resp.status();
        if !http_status.is_success() {
            return self
                .handle_http_poll_error_response(http_status, resp)
                .await;
        }
        let data: Value = match resp.json().await {
            Ok(data) => data,
            Err(error) => {
                tracing::warn!("Telegram parse error: {error}");
                return Ok(PollOutcome::RetryAfterSeconds(TELEGRAM_POLL_RETRY_SECS));
            }
        };
        Self::handle_api_poll_response(&data)
    }

    async fn handle_http_poll_error_response(
        &self,
        status: StatusCode,
        resp: reqwest::Response,
    ) -> anyhow::Result<PollOutcome> {
        let body_text = resp.text().await.unwrap_or_default();
        let maybe_data = serde_json::from_str::<Value>(&body_text).ok();
        let description = maybe_data
            .as_ref()
            .and_then(|value| value.get("description"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(body_text.as_str());

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                anyhow::bail!("Telegram getUpdates HTTP error (status={status}): {description}");
            }
            StatusCode::CONFLICT => {
                tracing::warn!(
                    "Telegram polling conflict (HTTP 409): {description}. \
Ensure only one process is using this bot token."
                );
                Ok(PollOutcome::RetryAfterSeconds(
                    TELEGRAM_POLL_CONFLICT_RETRY_SECS,
                ))
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after_secs = maybe_data
                    .as_ref()
                    .and_then(telegram_api_error_retry_after_secs)
                    .unwrap_or(TELEGRAM_POLL_DEFAULT_RATE_LIMIT_RETRY_SECS)
                    .clamp(1, TELEGRAM_POLL_MAX_RATE_LIMIT_RETRY_SECS);
                tracing::warn!(
                    retry_after_secs,
                    "Telegram getUpdates HTTP 429 rate limited: {description}"
                );
                Ok(PollOutcome::RetryAfterSeconds(retry_after_secs))
            }
            _ => {
                tracing::warn!(
                    status = %status,
                    "Telegram getUpdates HTTP error: {description}"
                );
                Ok(PollOutcome::RetryAfterSeconds(TELEGRAM_POLL_RETRY_SECS))
            }
        }
    }

    fn handle_api_poll_response(data: &Value) -> anyhow::Result<PollOutcome> {
        let ok = data.get("ok").and_then(Value::as_bool).unwrap_or(true);
        if !ok {
            return Self::handle_api_poll_error(data);
        }
        let updates = data
            .get("result")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        Ok(PollOutcome::Updates(updates))
    }

    fn handle_api_poll_error(data: &Value) -> anyhow::Result<PollOutcome> {
        let error_code = data
            .get("error_code")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let description = data
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("unknown Telegram API error");

        match error_code {
            401 | 403 => {
                anyhow::bail!("Telegram getUpdates API error (code={error_code}): {description}",);
            }
            409 => {
                tracing::warn!(
                    "Telegram polling conflict (409): {description}. \
Ensure only one process is using this bot token."
                );
                Ok(PollOutcome::RetryAfterSeconds(
                    TELEGRAM_POLL_CONFLICT_RETRY_SECS,
                ))
            }
            429 => {
                let retry_after_secs = telegram_api_error_retry_after_secs(data)
                    .unwrap_or(TELEGRAM_POLL_DEFAULT_RATE_LIMIT_RETRY_SECS)
                    .clamp(1, TELEGRAM_POLL_MAX_RATE_LIMIT_RETRY_SECS);
                tracing::warn!(
                    retry_after_secs,
                    "Telegram getUpdates rate limited (429): {description}"
                );
                Ok(PollOutcome::RetryAfterSeconds(retry_after_secs))
            }
            _ => {
                tracing::warn!(
                    "Telegram getUpdates API error (code={}): {description}",
                    error_code
                );
                Ok(PollOutcome::RetryAfterSeconds(TELEGRAM_POLL_RETRY_SECS))
            }
        }
    }

    async fn process_polled_updates(
        &self,
        tx: &mpsc::Sender<ChannelMessage>,
        updates: &[Value],
        offset: &mut i64,
    ) -> bool {
        for update in updates {
            if let Some(uid) = update.get("update_id").and_then(Value::as_i64) {
                *offset = uid + 1;
            }
            let Some(msg) = self.parse_update_message(update) else {
                continue;
            };
            let _ = self.send_chat_action(&msg.recipient, "typing").await;
            if tx.send(msg).await.is_err() {
                return false;
            }
        }
        true
    }

    pub(super) async fn health_probe(&self) -> bool {
        match tokio::time::timeout(
            Duration::from_secs(5),
            self.client.get(self.api_url("getMe")).send(),
        )
        .await
        {
            Ok(Ok(resp)) => resp.status().is_success(),
            _ => false,
        }
    }
}
