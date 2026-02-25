use std::sync::Arc;

use omni_agent::{
    DEFAULT_REDIS_KEY_PREFIX, RuntimeSettings, TelegramCommandAdminRule,
    TelegramControlCommandPolicy, TelegramSlashCommandPolicy, WebhookDedupBackend,
    WebhookDedupConfig, build_telegram_acl_overrides,
    run_telegram_webhook_with_control_command_policy, run_telegram_with_control_command_policy,
};

use crate::cli::{TelegramChannelMode, WebhookDedupBackendMode};
use crate::resolve::{
    resolve_channel_mode, resolve_dedup_backend, resolve_positive_u64, resolve_string,
};
use crate::runtime_agent_factory::build_agent;

use super::ChannelCommandRequest;
use super::common::{log_control_command_allow_override, log_slash_command_allow_override};

#[allow(clippy::similar_names)]
pub(super) async fn run_telegram_channel_command(
    req: ChannelCommandRequest,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let ChannelCommandRequest {
        bot_token,
        mcp_config,
        mode,
        webhook_bind,
        webhook_path,
        webhook_secret_token,
        webhook_dedup_backend,
        valkey_url,
        webhook_dedup_ttl_secs,
        webhook_dedup_key_prefix,
        ..
    } = req;

    let acl_overrides = build_telegram_acl_overrides(runtime_settings)?;
    let channel_mode = resolve_channel_mode(mode, runtime_settings.telegram.mode.as_deref());
    let allowed_users = acl_overrides.allowed_users;
    let allowed_groups = acl_overrides.allowed_groups;
    let admin_users = acl_overrides.admin_users;
    let control_command_allow_from = acl_overrides.control_command_allow_from;
    let control_command_rules = acl_overrides.control_command_rules;
    let slash_command_allow_from = acl_overrides.slash_command_allow_from;
    let slash_session_status_allow_from = acl_overrides.slash_session_status_allow_from;
    let slash_session_budget_allow_from = acl_overrides.slash_session_budget_allow_from;
    let slash_session_memory_allow_from = acl_overrides.slash_session_memory_allow_from;
    let slash_session_feedback_allow_from = acl_overrides.slash_session_feedback_allow_from;
    let slash_job_allow_from = acl_overrides.slash_job_allow_from;
    let slash_jobs_allow_from = acl_overrides.slash_jobs_allow_from;
    let slash_bg_allow_from = acl_overrides.slash_bg_allow_from;
    let webhook_bind = resolve_string(
        webhook_bind,
        "OMNI_AGENT_TELEGRAM_WEBHOOK_BIND",
        runtime_settings.telegram.webhook_bind.as_deref(),
        "127.0.0.1:8081",
    );
    let webhook_path = resolve_string(
        webhook_path,
        "OMNI_AGENT_TELEGRAM_WEBHOOK_PATH",
        runtime_settings.telegram.webhook_path.as_deref(),
        "/telegram/webhook",
    );
    let dedup_backend = resolve_dedup_backend(
        webhook_dedup_backend,
        runtime_settings.telegram.webhook_dedup_backend.as_deref(),
    );
    let webhook_dedup_ttl_secs = resolve_positive_u64(
        webhook_dedup_ttl_secs,
        "OMNI_AGENT_TELEGRAM_WEBHOOK_DEDUP_TTL_SECS",
        runtime_settings.telegram.webhook_dedup_ttl_secs,
        600,
    );
    let webhook_dedup_key_prefix = resolve_string(
        webhook_dedup_key_prefix,
        "OMNI_AGENT_TELEGRAM_WEBHOOK_DEDUP_KEY_PREFIX",
        runtime_settings
            .telegram
            .webhook_dedup_key_prefix
            .as_deref(),
        DEFAULT_REDIS_KEY_PREFIX,
    );
    let token = bot_token
        .or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok())
        .ok_or_else(|| anyhow::anyhow!("--bot-token or TELEGRAM_BOT_TOKEN required"))?;
    let secret_token = resolve_webhook_secret_token(channel_mode, webhook_secret_token)?;
    let dedup_config = build_webhook_dedup_config(
        dedup_backend,
        valkey_url,
        webhook_dedup_ttl_secs,
        webhook_dedup_key_prefix,
        runtime_settings,
    )?;

    run_telegram_channel_mode(
        token,
        allowed_users,
        allowed_groups,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        slash_command_allow_from,
        slash_session_status_allow_from,
        slash_session_budget_allow_from,
        slash_session_memory_allow_from,
        slash_session_feedback_allow_from,
        slash_job_allow_from,
        slash_jobs_allow_from,
        slash_bg_allow_from,
        mcp_config,
        channel_mode,
        webhook_bind,
        webhook_path,
        secret_token,
        dedup_config,
        runtime_settings,
    )
    .await
}

fn resolve_webhook_secret_token(
    channel_mode: TelegramChannelMode,
    cli_secret: Option<String>,
) -> anyhow::Result<Option<String>> {
    let secret = cli_secret
        .or_else(|| std::env::var("TELEGRAM_WEBHOOK_SECRET").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if matches!(channel_mode, TelegramChannelMode::Webhook) && secret.is_none() {
        return Err(anyhow::anyhow!(
            "webhook mode requires TELEGRAM_WEBHOOK_SECRET (or --webhook-secret-token)"
        ));
    }
    Ok(secret)
}

#[allow(clippy::similar_names, clippy::too_many_arguments)]
async fn run_telegram_channel_mode(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    admin_users: Vec<String>,
    control_command_allow_from: Option<Vec<String>>,
    control_command_rules: Vec<TelegramCommandAdminRule>,
    slash_command_allow_from: Option<Vec<String>>,
    slash_session_status_allow_from: Option<Vec<String>>,
    slash_session_budget_allow_from: Option<Vec<String>>,
    slash_session_memory_allow_from: Option<Vec<String>>,
    slash_session_feedback_allow_from: Option<Vec<String>>,
    slash_job_allow_from: Option<Vec<String>>,
    slash_jobs_allow_from: Option<Vec<String>>,
    slash_bg_allow_from: Option<Vec<String>>,
    mcp_config_path: std::path::PathBuf,
    mode: TelegramChannelMode,
    webhook_bind: String,
    webhook_path: String,
    webhook_secret_token: Option<String>,
    webhook_dedup_config: WebhookDedupConfig,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<()> {
    let agent = Arc::new(build_agent(&mcp_config_path, runtime_settings).await?);
    let users = allowed_users;
    let groups = allowed_groups;
    let admins = admin_users;
    let control_command_allow_from_entries = control_command_allow_from;
    log_control_command_allow_override("telegram", &control_command_allow_from_entries);
    let slash_command_allow_from_entries = slash_command_allow_from;
    log_slash_command_allow_override("telegram", &slash_command_allow_from_entries);
    let slash_command_policy = TelegramSlashCommandPolicy {
        slash_command_allow_from: slash_command_allow_from_entries,
        session_status_allow_from: slash_session_status_allow_from,
        session_budget_allow_from: slash_session_budget_allow_from,
        session_memory_allow_from: slash_session_memory_allow_from,
        session_feedback_allow_from: slash_session_feedback_allow_from,
        job_status_allow_from: slash_job_allow_from,
        jobs_summary_allow_from: slash_jobs_allow_from,
        background_submit_allow_from: slash_bg_allow_from,
    };
    let control_command_policy = TelegramControlCommandPolicy::new(
        admins,
        control_command_allow_from_entries,
        control_command_rules,
    )
    .with_slash_command_policy(slash_command_policy);
    if users.is_empty() && groups.is_empty() {
        tracing::warn!(
            "Telegram ACL allowlist is empty; all inbound will be rejected. \
             Configure `telegram.acl.allow.users` or `telegram.acl.allow.groups` to allow traffic."
        );
    }
    match mode {
        TelegramChannelMode::Polling => {
            run_telegram_with_control_command_policy(
                Arc::clone(&agent),
                bot_token,
                users,
                groups,
                control_command_policy.clone(),
            )
            .await
        }
        TelegramChannelMode::Webhook => {
            run_telegram_webhook_with_control_command_policy(
                Arc::clone(&agent),
                bot_token,
                users,
                groups,
                control_command_policy,
                &webhook_bind,
                &webhook_path,
                webhook_secret_token,
                webhook_dedup_config,
            )
            .await
        }
    }
}

fn build_webhook_dedup_config(
    backend_mode: WebhookDedupBackendMode,
    valkey_url: Option<String>,
    ttl_secs: u64,
    key_prefix: String,
    runtime_settings: &RuntimeSettings,
) -> anyhow::Result<WebhookDedupConfig> {
    let backend = match backend_mode {
        WebhookDedupBackendMode::Memory => WebhookDedupBackend::Memory,
        WebhookDedupBackendMode::Valkey => {
            let url = valkey_url
                .or_else(|| runtime_settings.session.valkey_url.clone())
                .or_else(|| std::env::var("VALKEY_URL").ok())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "valkey dedup backend requires valkey url (explicit --valkey-url, session.valkey_url, or VALKEY_URL)"
                    )
                })?;
            if url.trim().is_empty() {
                return Err(anyhow::anyhow!(
                    "valkey dedup backend requires a non-empty URL"
                ));
            }
            WebhookDedupBackend::Redis { url, key_prefix }
        }
    };

    Ok(WebhookDedupConfig { backend, ttl_secs })
}
