use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::{NotificationProvider, recipient_is_telegram_chat_id, recipient_target_for};

const MARKDOWN_V2_PREFIX: &str = "[markdown_v2]\n";

/// Telegram bot API notification provider.
pub struct TelegramProvider {
    client: Client,
}

impl TelegramProvider {
    /// Create a telegram provider.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl NotificationProvider for TelegramProvider {
    fn name(&self) -> &'static str {
        "telegram"
    }

    fn supports(&self, recipient: &str) -> bool {
        recipient_target_for(recipient, &["telegram", "tg"]).is_some()
            || recipient_is_telegram_chat_id(recipient)
    }

    async fn send(&self, recipient: &str, content: &str) -> Result<()> {
        let chat_id = recipient_target_for(recipient, &["telegram", "tg"])
            .unwrap_or(recipient)
            .trim()
            .to_string();
        if chat_id.is_empty() {
            anyhow::bail!("telegram recipient chat id is empty");
        }

        let token = std::env::var("TELEGRAM_BOT_TOKEN")
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|raw| !raw.is_empty())
            .ok_or_else(|| anyhow::anyhow!("TELEGRAM_BOT_TOKEN is required"))?;
        let base = std::env::var("OMNI_AGENT_TELEGRAM_API_BASE_URL")
            .ok()
            .map(|raw| raw.trim().trim_end_matches('/').to_string())
            .filter(|raw| !raw.is_empty())
            .unwrap_or_else(|| "https://api.telegram.org".to_string());
        let url = format!("{base}/bot{token}/sendMessage");
        let (text, parse_mode) = if let Some(markdown_v2) = content.strip_prefix(MARKDOWN_V2_PREFIX)
        {
            (markdown_v2, "MarkdownV2")
        } else {
            (content, "HTML")
        };

        self.client
            .post(url)
            .json(&json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": parse_mode,
            }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
