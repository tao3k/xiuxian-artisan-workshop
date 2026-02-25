use std::path::PathBuf;
use std::sync::Arc;

use omni_agent::{
    DiscordCommandAdminRule, DiscordControlCommandPolicy, DiscordIngressRunRequest,
    DiscordRuntimeConfig, DiscordSessionPartition, DiscordSlashCommandPolicy, RuntimeSettings,
    build_discord_acl_overrides, run_discord_gateway, run_discord_ingress,
};

use crate::cli::DiscordRuntimeMode;
use crate::resolve::{
    resolve_discord_runtime_mode, resolve_positive_u64, resolve_positive_usize, resolve_string,
};
use crate::runtime_agent_factory::build_agent;

use super::ChannelCommandRequest;
use super::common::{log_control_command_allow_override, log_slash_command_allow_override};

const DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY: usize = 512;
const DISCORD_DEFAULT_TURN_TIMEOUT_SECS: u64 = 120;
const DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES: usize = 16;
const DISCORD_DEFAULT_INGRESS_BIND: &str = "0.0.0.0:18082";
const DISCORD_DEFAULT_INGRESS_PATH: &str = "/discord/ingress";

struct DiscordRuntimeLaunchConfig {
    bot_token: String,
    mcp_config_path: PathBuf,
    runtime_mode: DiscordRuntimeMode,
    runtime_config: DiscordRuntimeConfig,
    ingress_bind: String,
    ingress_path: String,
    ingress_secret_token: Option<String>,
}

struct DiscordAclLaunchConfig {
    allowed_users: Vec<String>,
    allowed_guilds: Vec<String>,
    admin_users: Vec<String>,
    control_command_allow_from: Option<Vec<String>>,
    control_command_rules: Vec<DiscordCommandAdminRule>,
    slash_command_policy: DiscordSlashCommandPolicy,
}

struct DiscordChannelModeRequest {
    runtime: DiscordRuntimeLaunchConfig,
    acl: DiscordAclLaunchConfig,
}

pub(super) async fn run_discord_channel_command(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let runtime = resolve_discord_runtime_launch_config(req, runtime_settings)?;
    let acl = resolve_discord_acl_launch_config(runtime_settings)?;
    run_discord_channel_mode(DiscordChannelModeRequest { runtime, acl }, runtime_settings).await
}

fn resolve_discord_runtime_launch_config(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<DiscordRuntimeLaunchConfig> {
    let ChannelCommandRequest {
        bot_token,
        mcp_config,
        session_partition,
        inbound_queue_capacity,
        turn_timeout_secs,
        discord_runtime_mode,
        ..
    } = req;

    let bot_token = bot_token
        .or_else(|| std::env::var("DISCORD_BOT_TOKEN").ok())
        .ok_or_else(|| anyhow::anyhow!("--bot-token or DISCORD_BOT_TOKEN required"))?;
    let raw_partition = resolve_string(
        session_partition,
        "OMNI_AGENT_DISCORD_SESSION_PARTITION",
        runtime_settings.discord.session_partition.as_deref(),
        "guild_channel_user",
    );
    let session_partition = raw_partition
        .parse::<DiscordSessionPartition>()
        .map_err(|_| anyhow::anyhow!("invalid discord session partition mode: {raw_partition}"))?;
    let inbound_queue_capacity = resolve_positive_usize(
        inbound_queue_capacity,
        "OMNI_AGENT_DISCORD_INBOUND_QUEUE_CAPACITY",
        runtime_settings.discord.inbound_queue_capacity,
        DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY,
    );
    let turn_timeout_secs = resolve_positive_u64(
        turn_timeout_secs,
        "OMNI_AGENT_DISCORD_TURN_TIMEOUT_SECS",
        runtime_settings.discord.turn_timeout_secs,
        DISCORD_DEFAULT_TURN_TIMEOUT_SECS,
    );
    let foreground_max_in_flight_messages = resolve_positive_usize(
        None,
        "OMNI_AGENT_DISCORD_FOREGROUND_MAX_IN_FLIGHT_MESSAGES",
        runtime_settings.discord.foreground_max_in_flight_messages,
        DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES,
    );
    let runtime_mode = resolve_discord_runtime_mode(
        discord_runtime_mode,
        runtime_settings.discord.runtime_mode.as_deref(),
    );
    let ingress_bind = resolve_string(
        None,
        "OMNI_AGENT_DISCORD_INGRESS_BIND",
        runtime_settings.discord.ingress_bind.as_deref(),
        DISCORD_DEFAULT_INGRESS_BIND,
    );
    let ingress_path = resolve_string(
        None,
        "OMNI_AGENT_DISCORD_INGRESS_PATH",
        runtime_settings.discord.ingress_path.as_deref(),
        DISCORD_DEFAULT_INGRESS_PATH,
    );
    let ingress_secret_token = std::env::var("OMNI_AGENT_DISCORD_INGRESS_SECRET_TOKEN")
        .ok()
        .or_else(|| runtime_settings.discord.ingress_secret_token.clone())
        .and_then(|secret| normalize_non_empty_secret(&secret));

    Ok(DiscordRuntimeLaunchConfig {
        bot_token,
        mcp_config_path: mcp_config,
        runtime_mode,
        runtime_config: DiscordRuntimeConfig {
            session_partition,
            inbound_queue_capacity,
            turn_timeout_secs,
            foreground_max_in_flight_messages,
        },
        ingress_bind,
        ingress_path,
        ingress_secret_token,
    })
}

fn resolve_discord_acl_launch_config(
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<DiscordAclLaunchConfig> {
    let acl_overrides = build_discord_acl_overrides(runtime_settings)?;
    let allowed_users = acl_overrides.allowed_users;
    let allowed_guilds = acl_overrides.allowed_guilds;
    let admin_users = acl_overrides
        .admin_users
        .unwrap_or_else(|| allowed_users.clone());
    let control_command_allow_from = acl_overrides.control_command_allow_from;
    let slash_command_allow_from = acl_overrides.slash_command_allow_from;
    let slash_command_policy = DiscordSlashCommandPolicy {
        slash_command_allow_from: slash_command_allow_from.clone(),
        session_status_allow_from: acl_overrides.slash_session_status_allow_from,
        session_budget_allow_from: acl_overrides.slash_session_budget_allow_from,
        session_memory_allow_from: acl_overrides.slash_session_memory_allow_from,
        session_feedback_allow_from: acl_overrides.slash_session_feedback_allow_from,
        job_status_allow_from: acl_overrides.slash_job_allow_from,
        jobs_summary_allow_from: acl_overrides.slash_jobs_allow_from,
        background_submit_allow_from: acl_overrides.slash_bg_allow_from,
    };
    log_control_command_allow_override("discord", &control_command_allow_from);
    log_slash_command_allow_override("discord", &slash_command_allow_from);

    Ok(DiscordAclLaunchConfig {
        allowed_users,
        allowed_guilds,
        admin_users,
        control_command_allow_from,
        control_command_rules: acl_overrides.control_command_rules,
        slash_command_policy,
    })
}

fn normalize_non_empty_secret(secret: &str) -> Option<String> {
    let trimmed = secret.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

async fn run_discord_channel_mode(
    request: DiscordChannelModeRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let DiscordChannelModeRequest { runtime, acl } = request;
    let DiscordRuntimeLaunchConfig {
        bot_token,
        mcp_config_path,
        runtime_mode,
        runtime_config,
        ingress_bind,
        ingress_path,
        ingress_secret_token,
    } = runtime;
    let DiscordAclLaunchConfig {
        allowed_users,
        allowed_guilds,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        slash_command_policy,
    } = acl;

    let agent = Arc::new(build_agent(&mcp_config_path, runtime_settings).await?);
    let control_command_policy = DiscordControlCommandPolicy::new(
        admin_users,
        control_command_allow_from,
        control_command_rules,
    )
    .with_slash_command_policy(slash_command_policy);

    if allowed_users.is_empty() && allowed_guilds.is_empty() {
        tracing::warn!(
            "Discord ACL allowlist is empty; all inbound will be rejected. \
             Configure `discord.acl.allow.users` or `discord.acl.allow.guilds` to allow traffic."
        );
    }

    match runtime_mode {
        DiscordRuntimeMode::Gateway => {
            run_discord_gateway(
                Arc::clone(&agent),
                bot_token,
                allowed_users,
                allowed_guilds,
                control_command_policy,
                runtime_config,
            )
            .await
        }
        DiscordRuntimeMode::Ingress => {
            run_discord_ingress(
                Arc::clone(&agent),
                DiscordIngressRunRequest {
                    bot_token,
                    allowed_users,
                    allowed_guilds,
                    control_command_policy,
                    bind_addr: ingress_bind,
                    ingress_path,
                    secret_token: ingress_secret_token,
                },
                runtime_config,
            )
            .await
        }
    }
}
