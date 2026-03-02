use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::{NotificationProvider, recipient_target_for};

/// Discord channel API notification provider.
pub struct DiscordProvider {
    client: Client,
}

impl DiscordProvider {
    /// Create a discord provider.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl NotificationProvider for DiscordProvider {
    fn name(&self) -> &'static str {
        "discord"
    }

    fn supports(&self, recipient: &str) -> bool {
        recipient_target_for(recipient, &["discord"]).is_some()
    }

    async fn send(&self, recipient: &str, content: &str) -> Result<()> {
        let channel_id = recipient_target_for(recipient, &["discord"])
            .ok_or_else(|| anyhow::anyhow!("discord recipient must be `discord:<channel_id>`"))?;
        let token = std::env::var("DISCORD_BOT_TOKEN")
            .ok()
            .map(|raw| raw.trim().to_string())
            .filter(|raw| !raw.is_empty())
            .ok_or_else(|| anyhow::anyhow!("DISCORD_BOT_TOKEN is required"))?;
        let base = std::env::var("OMNI_AGENT_DISCORD_API_BASE_URL")
            .ok()
            .map(|raw| raw.trim().trim_end_matches('/').to_string())
            .filter(|raw| !raw.is_empty())
            .unwrap_or_else(|| "https://discord.com/api/v10".to_string());
        let url = format!("{base}/channels/{channel_id}/messages");

        self.client
            .post(url)
            .header("Authorization", format!("Bot {token}"))
            .json(&json!({ "content": content }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
