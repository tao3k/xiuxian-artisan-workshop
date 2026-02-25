use std::sync::Arc;
use std::time::Instant;

use serenity::all::Message;
use serenity::client::{Context, EventHandler};
use tokio::sync::mpsc;

use super::super::super::channel::DiscordChannel;
use crate::channels::traits::ChannelMessage;

pub(super) struct DiscordGatewayEventHandler {
    pub(super) channel: Arc<DiscordChannel>,
    pub(super) tx: mpsc::Sender<ChannelMessage>,
}

#[serenity::async_trait]
impl EventHandler for DiscordGatewayEventHandler {
    async fn message(&self, _ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }
        let payload = match serde_json::to_value(&message) {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(error = %error, "failed to serialize discord gateway message");
                return;
            }
        };
        let Some(parsed) = self.channel.parse_gateway_message(&payload) else {
            return;
        };
        let session_key = parsed.session_key.clone();
        let recipient = parsed.recipient.clone();
        let send_started = Instant::now();
        if self.tx.send(parsed).await.is_err() {
            tracing::warn!("discord inbound queue unavailable");
            return;
        }
        let send_wait_ms = u64::try_from(send_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if send_wait_ms >= 50 {
            tracing::warn!(
                event = "discord.gateway.inbound_queue_wait",
                wait_ms = send_wait_ms,
                session_key = %session_key,
                recipient = %recipient,
                "discord gateway waited on inbound queue send"
            );
        }
    }
}
