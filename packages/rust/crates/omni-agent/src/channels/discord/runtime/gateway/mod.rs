use std::sync::Arc;

use anyhow::Result;
use serenity::all::GatewayIntents;
use serenity::client::Client;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use super::super::channel::{DiscordChannel, DiscordControlCommandPolicy};
use super::DiscordRuntimeConfig;
use super::foreground::build_foreground_runtime;
use super::telemetry::snapshot_interval_from_env;
use crate::agent::Agent;
use crate::channels::traits::{Channel, ChannelMessage};

mod event_handler;
mod loop_control;

use event_handler::DiscordGatewayEventHandler;
use loop_control::drive_gateway_runtime_loop;

/// Run Discord channel via serenity gateway event stream.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub async fn run_discord_gateway(
    agent: Arc<Agent>,
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    control_command_policy: DiscordControlCommandPolicy,
    runtime_config: DiscordRuntimeConfig,
) -> Result<()> {
    let DiscordRuntimeConfig {
        session_partition,
        inbound_queue_capacity,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
    } = runtime_config;
    let channel = Arc::new(
        DiscordChannel::new_with_partition_and_control_command_policy(
            bot_token.clone(),
            allowed_users,
            allowed_guilds,
            control_command_policy,
            session_partition,
        )?,
    );
    let channel_for_send: Arc<dyn Channel> = channel.clone();
    let (tx, mut inbound_rx) = mpsc::channel::<ChannelMessage>(inbound_queue_capacity);
    let inbound_snapshot_tx = tx.clone();
    let (mut runtime, mut completion_rx) = build_foreground_runtime(
        agent,
        channel_for_send,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
    );
    let mut snapshot_tick = build_snapshot_tick().await;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;
    let handler = DiscordGatewayEventHandler {
        channel: Arc::clone(&channel),
        tx,
    };
    let mut client = Client::builder(bot_token, intents)
        .event_handler(handler)
        .await?;
    let shard_manager = client.shard_manager.clone();
    let mut gateway_task = tokio::spawn(async move { client.start().await });

    print_gateway_banner(
        &channel,
        inbound_queue_capacity,
        foreground_max_in_flight_messages,
        turn_timeout_secs,
    );

    let shutdown_requested = drive_gateway_runtime_loop(
        &mut runtime,
        &mut inbound_rx,
        &mut completion_rx,
        &inbound_snapshot_tx,
        inbound_queue_capacity,
        &mut snapshot_tick,
        &mut gateway_task,
    )
    .await;

    runtime.abort_and_drain_foreground_tasks().await;

    if shutdown_requested {
        shard_manager.shutdown_all().await;
        if let Err(error) = gateway_task.await {
            tracing::error!("discord gateway task join error during shutdown: {error}");
        }
    }

    Ok(())
}

async fn build_snapshot_tick() -> Option<tokio::time::Interval> {
    let mut snapshot_tick = snapshot_interval_from_env().map(|period| {
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        interval
    });
    if let Some(interval) = snapshot_tick.as_mut() {
        let _ = interval.tick().await;
    }
    snapshot_tick
}

fn print_gateway_banner(
    channel: &DiscordChannel,
    inbound_queue_capacity: usize,
    foreground_max_in_flight_messages: usize,
    turn_timeout_secs: u64,
) {
    println!("Discord gateway connected (Ctrl+C to stop)");
    println!("Discord session partition: {}", channel.session_partition());
    println!(
        "Discord foreground config: inbound_queue={inbound_queue_capacity} max_in_flight={foreground_max_in_flight_messages} timeout={turn_timeout_secs}s"
    );
    println!("Background commands: /bg <prompt>, /job <id> [json], /jobs [json]");
    println!(
        "Session commands: /help [json], /session [json], /session budget [json], /session memory [json], /session feedback up|down [json], /session partition [mode|on|off] [json], /session admin [list|set|add|remove|clear] [json], /session inject [status|clear|<qa>...</qa>] [json], /feedback up|down [json], /reset, /clear, /resume, /resume drop, /stop"
    );
}
