use crate::agent::Agent;
use crate::channels::managed_runtime::turn::{
    ForegroundTurnOutcome, ForegroundTurnRequest, run_foreground_turn_with_interrupt,
};
use crate::channels::traits::{Channel, ChannelMessage};
use std::sync::Arc;
use tokio::sync::watch;

use super::preview::log_preview;

pub(super) struct ForegroundTurnInput<'a> {
    pub(super) recipient: &'a str,
    pub(super) session_id: &'a str,
    pub(super) content: &'a str,
    pub(super) turn_timeout_secs: u64,
    pub(super) interrupt_rx: watch::Receiver<u64>,
    pub(super) interrupt_generation: u64,
}

pub(super) async fn run_foreground_turn_with_typing(
    channel: &dyn Channel,
    agent: Arc<Agent>,
    input: ForegroundTurnInput<'_>,
) -> ForegroundTurnOutcome {
    if let Err(error) = channel.start_typing(input.recipient).await {
        tracing::debug!("discord: failed to start typing: {error}");
    }

    let result = run_foreground_turn_with_interrupt(ForegroundTurnRequest {
        agent,
        session_id: input.session_id.to_string(),
        content: input.content.to_string(),
        timeout_secs: input.turn_timeout_secs,
        timeout_reply: format!("Request timed out after {}s.", input.turn_timeout_secs),
        interrupt_rx: input.interrupt_rx,
        interrupt_generation: input.interrupt_generation,
        interrupted_reply: "Request interrupted by a newer instruction.".to_string(),
    })
    .await;

    if let Err(error) = channel.stop_typing(input.recipient).await {
        tracing::debug!("discord: failed to stop typing: {error}");
    }
    result
}

pub(super) fn render_foreground_turn_reply(
    result: ForegroundTurnOutcome,
    msg: &ChannelMessage,
    turn_timeout_secs: u64,
) -> String {
    match result {
        ForegroundTurnOutcome::Succeeded(output) => output,
        ForegroundTurnOutcome::Failed {
            reply,
            error_chain,
            error_kind,
        } => {
            tracing::error!(
                event = "discord.foreground.turn.failed",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                error_kind,
                error = %error_chain,
                "discord foreground turn failed"
            );
            reply
        }
        ForegroundTurnOutcome::TimedOut { reply } => {
            tracing::warn!(
                event = "discord.foreground.turn.timeout",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                timeout_secs = turn_timeout_secs,
                "discord foreground turn timed out"
            );
            reply
        }
        ForegroundTurnOutcome::Interrupted { reply } => {
            tracing::warn!(
                event = "discord.foreground.turn.interrupted",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                "discord foreground turn interrupted"
            );
            reply
        }
    }
}

pub(super) async fn send_discord_reply(channel: &dyn Channel, msg: &ChannelMessage, reply: &str) {
    match channel.send(reply, &msg.recipient).await {
        Ok(()) => tracing::info!(
            r#"discord → Bot: "{preview}""#,
            preview = log_preview(reply)
        ),
        Err(error) => tracing::warn!("discord: failed to send reply: {error}"),
    }
}
