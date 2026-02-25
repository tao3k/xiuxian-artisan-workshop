//! omni-agent CLI: gateway, stdio, or repl mode.
//!
//! MCP servers from mcp.json only (default `.mcp.json`). Override with `--mcp-config <path>`.
//!
//! Logging: set `RUST_LOG=omni_agent=info` (or `warn`, `debug`) to see agent logs on stderr.

#![recursion_limit = "256"]

mod cli;
mod nodes;
mod resolve;
mod runtime_agent_factory;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use omni_agent::{RuntimeSettings, load_runtime_settings, set_config_home_override};

use crate::cli::{Cli, Command};
use crate::nodes::{
    ChannelCommandRequest, run_channel_command, run_gateway_mode, run_repl_mode, run_schedule_mode,
    run_stdio_mode,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if let Some(conf_dir) = cli.conf.clone() {
        set_config_home_override(conf_dir);
    }
    let runtime_settings = load_runtime_settings();
    init_tracing(&cli);
    dispatch_command(cli.command, &runtime_settings).await
}

fn init_tracing(cli: &Cli) {
    // Initialize tracing: RUST_LOG overrides; --verbose on channel => debug; else info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let verbose = matches!(&cli.command, Command::Channel { verbose: true, .. });
        EnvFilter::new(if verbose {
            "omni_agent=debug"
        } else {
            "omni_agent=info"
        })
    });
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

async fn dispatch_command(
    command: Command,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    match command {
        Command::Gateway {
            bind,
            turn_timeout,
            max_concurrent,
            mcp_config,
        } => {
            run_gateway_mode(
                bind,
                turn_timeout,
                max_concurrent,
                mcp_config,
                runtime_settings,
            )
            .await
        }
        Command::Stdio {
            session_id,
            mcp_config,
        } => run_stdio_mode(session_id, mcp_config, runtime_settings).await,
        Command::Repl {
            query,
            session_id,
            mcp_config,
        } => run_repl_mode(query, session_id, mcp_config, runtime_settings).await,
        Command::Schedule {
            prompt,
            interval_secs,
            max_runs,
            schedule_id,
            session_prefix,
            recipient,
            wait_for_completion_secs,
            mcp_config,
        } => {
            run_schedule_mode(
                prompt,
                interval_secs,
                max_runs,
                schedule_id,
                session_prefix,
                recipient,
                wait_for_completion_secs,
                mcp_config,
                runtime_settings,
            )
            .await
        }
        Command::Channel {
            provider,
            bot_token,
            mcp_config,
            mode,
            webhook_bind,
            webhook_path,
            webhook_secret_token,
            session_partition,
            inbound_queue_capacity,
            turn_timeout_secs,
            discord_runtime_mode,
            webhook_dedup_backend,
            valkey_url,
            webhook_dedup_ttl_secs,
            webhook_dedup_key_prefix,
            verbose: _,
        } => {
            run_channel_command(
                ChannelCommandRequest {
                    provider,
                    bot_token,
                    mcp_config,
                    mode,
                    webhook_bind,
                    webhook_path,
                    webhook_secret_token,
                    session_partition,
                    inbound_queue_capacity,
                    turn_timeout_secs,
                    discord_runtime_mode,
                    webhook_dedup_backend,
                    valkey_url,
                    webhook_dedup_ttl_secs,
                    webhook_dedup_key_prefix,
                },
                runtime_settings,
            )
            .await
        }
    }
}
