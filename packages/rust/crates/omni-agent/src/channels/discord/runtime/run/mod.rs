use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use super::super::channel::{DiscordChannel, DiscordControlCommandPolicy};
use super::DiscordRuntimeConfig;
use super::foreground::build_foreground_runtime;
use super::ingress::{
    DiscordIngressApp, build_discord_ingress_app_with_partition_and_control_command_policy,
};
use super::telemetry::snapshot_interval_from_env;
use crate::agent::Agent;
use crate::channels::traits::{Channel, ChannelMessage};

mod loop_control;

use loop_control::drive_ingress_runtime_loop;

/// Parameters to run Discord HTTP ingress runtime.
#[derive(Debug)]
pub struct DiscordIngressRunRequest {
    /// Bot token used by outbound Discord API calls.
    pub bot_token: String,
    /// Optional allowlist of user ids.
    pub allowed_users: Vec<String>,
    /// Optional allowlist of guild ids.
    pub allowed_guilds: Vec<String>,
    /// Policy for control and slash managed commands.
    pub control_command_policy: DiscordControlCommandPolicy,
    /// TCP address for ingress listener.
    pub bind_addr: String,
    /// HTTP path for ingress endpoint.
    pub ingress_path: String,
    /// Optional shared secret token for ingress validation.
    pub secret_token: Option<String>,
}

/// Run Discord channel via HTTP ingress endpoint.
///
/// # Errors
/// Returns an error when channel/runtime initialization fails.
pub async fn run_discord_ingress(
    agent: Arc<Agent>,
    request: DiscordIngressRunRequest,
    runtime_config: DiscordRuntimeConfig,
) -> Result<()> {
    let DiscordIngressRunRequest {
        bot_token,
        allowed_users,
        allowed_guilds,
        control_command_policy,
        bind_addr,
        ingress_path,
        secret_token,
    } = request;
    let DiscordRuntimeConfig {
        session_partition,
        inbound_queue_capacity,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
    } = runtime_config;

    let (tx, mut inbound_rx) = mpsc::channel::<ChannelMessage>(inbound_queue_capacity);
    let inbound_snapshot_tx = tx.clone();
    let ingress = build_discord_ingress_app_with_partition_and_control_command_policy(
        bot_token,
        allowed_users,
        allowed_guilds,
        control_command_policy,
        &ingress_path,
        secret_token,
        session_partition,
        tx,
    )?;
    let DiscordIngressApp { app, channel, path } = ingress;
    let channel_for_send: Arc<dyn Channel> = channel.clone();
    let (mut runtime, mut completion_rx) = build_foreground_runtime(
        agent,
        channel_for_send,
        turn_timeout_secs,
        foreground_max_in_flight_messages,
    );
    let mut snapshot_tick = build_snapshot_tick().await;
    let listener = TcpListener::bind(&bind_addr).await?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let mut ingress_server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    print_ingress_banner(
        &bind_addr,
        &path,
        &channel,
        inbound_queue_capacity,
        foreground_max_in_flight_messages,
        turn_timeout_secs,
    );

    drive_ingress_runtime_loop(
        &mut runtime,
        &mut inbound_rx,
        &mut completion_rx,
        &inbound_snapshot_tx,
        inbound_queue_capacity,
        &mut snapshot_tick,
        &mut ingress_server,
    )
    .await;

    runtime.abort_and_drain_foreground_tasks().await;

    let _ = shutdown_tx.send(());
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

fn print_ingress_banner(
    bind_addr: &str,
    path: &str,
    channel: &Arc<DiscordChannel>,
    inbound_queue_capacity: usize,
    foreground_max_in_flight_messages: usize,
    turn_timeout_secs: u64,
) {
    println!("Discord ingress listening on {bind_addr}{path} (Ctrl+C to stop)");
    println!("Discord session partition: {}", channel.session_partition());
    println!(
        "Discord foreground config: inbound_queue={inbound_queue_capacity} max_in_flight={foreground_max_in_flight_messages} timeout={turn_timeout_secs}s"
    );
    println!("Background commands: /bg <prompt>, /job <id> [json], /jobs [json]");
    println!(
        "Session commands: /help [json], /session [json], /session budget [json], /session memory [json], /session feedback up|down [json], /session partition [mode|on|off] [json], /session admin [list|set|add|remove|clear] [json], /session inject [status|clear|<qa>...</qa>] [json], /feedback up|down [json], /reset, /clear, /resume, /resume drop, /stop"
    );
}
