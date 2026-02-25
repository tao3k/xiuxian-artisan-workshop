use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use tokio::sync::mpsc;

use crate::channels::traits::ChannelMessage;

use super::super::channel::{DiscordChannel, DiscordControlCommandPolicy};
use super::super::session_partition::DiscordSessionPartition;

const DISCORD_INGRESS_SECRET_HEADER: &str = "x-omni-discord-ingress-token";

/// Built ingress components for Discord handler testing and runtime wiring.
pub struct DiscordIngressApp {
    /// Axum router that serves Discord ingress endpoint.
    pub app: Router,
    /// Discord channel instance used by this ingress app.
    pub channel: Arc<DiscordChannel>,
    /// Normalized ingress route path.
    pub path: String,
}

/// Build a Discord ingress app.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub fn build_discord_ingress_app(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    ingress_path: &str,
    secret_token: Option<String>,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<DiscordIngressApp> {
    let admin_users = allowed_users.clone();
    build_discord_ingress_app_with_control_command_policy(
        bot_token,
        allowed_users,
        allowed_guilds,
        DiscordControlCommandPolicy::new(admin_users, None, Vec::new()),
        ingress_path,
        secret_token,
        tx,
    )
}

/// Build a Discord ingress app with explicit control-command policy.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub fn build_discord_ingress_app_with_control_command_policy(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    control_command_policy: DiscordControlCommandPolicy,
    ingress_path: &str,
    secret_token: Option<String>,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<DiscordIngressApp> {
    build_discord_ingress_app_with_partition_and_control_command_policy(
        bot_token,
        allowed_users,
        allowed_guilds,
        control_command_policy,
        ingress_path,
        secret_token,
        DiscordSessionPartition::from_env(),
        tx,
    )
}

/// Build a Discord ingress app with explicit session partition and control-command policy.
#[doc(hidden)]
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
#[allow(clippy::too_many_arguments)]
pub fn build_discord_ingress_app_with_partition_and_control_command_policy(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    control_command_policy: DiscordControlCommandPolicy,
    ingress_path: &str,
    secret_token: Option<String>,
    session_partition: DiscordSessionPartition,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<DiscordIngressApp> {
    let channel = Arc::new(
        DiscordChannel::new_with_partition_and_control_command_policy(
            bot_token,
            allowed_users,
            allowed_guilds,
            control_command_policy,
            session_partition,
        )?,
    );
    let ingress_state = DiscordIngressState {
        channel: Arc::clone(&channel),
        tx,
        secret_token,
    };

    let path = normalize_ingress_path(ingress_path);
    let app = Router::new()
        .route(&path, post(discord_ingress_handler))
        .with_state(ingress_state);

    Ok(DiscordIngressApp { app, channel, path })
}

#[derive(Clone)]
struct DiscordIngressState {
    channel: Arc<DiscordChannel>,
    tx: mpsc::Sender<ChannelMessage>,
    secret_token: Option<String>,
}

fn normalize_ingress_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        "/discord/ingress".to_string()
    } else if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

async fn discord_ingress_handler(
    State(state): State<DiscordIngressState>,
    headers: HeaderMap,
    Json(event): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Some(expected) = state.secret_token.as_deref() {
        let provided = headers
            .get(DISCORD_INGRESS_SECRET_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        if provided != expected {
            return Err((
                StatusCode::UNAUTHORIZED,
                "invalid discord ingress secret token".to_string(),
            ));
        }
    }

    match state.channel.parse_gateway_message(&event) {
        Some(msg) => {
            let session_key = msg.session_key.clone();
            let recipient = msg.recipient.clone();
            tracing::info!(
                channel = "discord",
                session_key = %session_key,
                recipient = %recipient,
                "discord ingress parsed message"
            );
            let send_started = Instant::now();
            if state.tx.send(msg).await.is_err() {
                tracing::error!("discord inbound queue unavailable");
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "discord inbound queue unavailable".to_string(),
                ));
            }
            let send_wait_ms =
                u64::try_from(send_started.elapsed().as_millis()).unwrap_or(u64::MAX);
            if send_wait_ms >= 50 {
                tracing::warn!(
                    event = "discord.ingress.inbound_queue_wait",
                    wait_ms = send_wait_ms,
                    session_key = %session_key,
                    recipient = %recipient,
                    "discord ingress waited on inbound queue send"
                );
            }
        }
        None => {
            tracing::debug!("discord ingress ignored event");
        }
    }

    Ok(StatusCode::OK)
}
