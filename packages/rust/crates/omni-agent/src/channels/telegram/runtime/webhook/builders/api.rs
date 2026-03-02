use anyhow::Result;
use tokio::sync::mpsc;

use super::super::app::TelegramWebhookApp;
use super::core::{self, TelegramWebhookCoreBuildRequest};
use crate::channels::telegram::idempotency::WebhookDedupConfig;
use crate::channels::telegram::session_partition::TelegramSessionPartition;
use crate::channels::telegram::{TelegramCommandAdminRule, TelegramControlCommandPolicy};
use crate::channels::traits::ChannelMessage;

/// Request to build a webhook app with explicit control-command policy.
pub struct TelegramWebhookControlPolicyBuildRequest {
    /// Telegram bot token.
    pub bot_token: String,
    /// Allowed sender user identifiers.
    pub allowed_users: Vec<String>,
    /// Allowed group identifiers.
    pub allowed_groups: Vec<String>,
    /// Structured control/slash command policy.
    pub control_command_policy: TelegramControlCommandPolicy,
    /// Webhook route path.
    pub webhook_path: String,
    /// Optional webhook secret token.
    pub secret_token: Option<String>,
    /// Webhook dedup backend configuration.
    pub dedup_config: WebhookDedupConfig,
    /// Inbound channel sender for runtime dispatch.
    pub tx: mpsc::Sender<ChannelMessage>,
}

/// Request to build a webhook app with explicit session partition and admin users.
pub struct TelegramWebhookPartitionBuildRequest {
    /// Telegram bot token.
    pub bot_token: String,
    /// Allowed sender user identifiers.
    pub allowed_users: Vec<String>,
    /// Allowed group identifiers.
    pub allowed_groups: Vec<String>,
    /// Fallback admin users for control/slash ACL.
    pub admin_users: Vec<String>,
    /// Webhook route path.
    pub webhook_path: String,
    /// Optional webhook secret token.
    pub secret_token: Option<String>,
    /// Webhook dedup backend configuration.
    pub dedup_config: WebhookDedupConfig,
    /// Explicit session partition strategy.
    pub session_partition: TelegramSessionPartition,
    /// Inbound channel sender for runtime dispatch.
    pub tx: mpsc::Sender<ChannelMessage>,
}

struct TelegramWebhookAdminUsersBuildRequest {
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    admin_users: Vec<String>,
    webhook_path: String,
    secret_token: Option<String>,
    dedup_config: WebhookDedupConfig,
    tx: mpsc::Sender<ChannelMessage>,
}

struct TelegramWebhookAdminRulesBuildRequest {
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    admin_users: Vec<String>,
    control_command_allow_from: Option<Vec<String>>,
    control_command_rules: Vec<TelegramCommandAdminRule>,
    webhook_path: String,
    secret_token: Option<String>,
    dedup_config: WebhookDedupConfig,
    tx: mpsc::Sender<ChannelMessage>,
}

struct TelegramWebhookPartitionPolicyBuildRequest {
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    control_command_policy: TelegramControlCommandPolicy,
    webhook_path: String,
    secret_token: Option<String>,
    dedup_config: WebhookDedupConfig,
    session_partition: TelegramSessionPartition,
    tx: mpsc::Sender<ChannelMessage>,
}

/// Build a Telegram webhook app with configured dedup backend.
///
/// # Errors
/// Returns an error when channel construction or webhook app assembly fails.
pub fn build_telegram_webhook_app(
    bot_token: String,
    allowed_users: Vec<String>,
    allowed_groups: Vec<String>,
    webhook_path: &str,
    secret_token: Option<String>,
    dedup_config: WebhookDedupConfig,
    tx: mpsc::Sender<ChannelMessage>,
) -> Result<TelegramWebhookApp> {
    build_telegram_webhook_app_with_admin_users(TelegramWebhookAdminUsersBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        admin_users: Vec::new(),
        webhook_path: webhook_path.to_string(),
        secret_token,
        dedup_config,
        tx,
    })
}

fn build_telegram_webhook_app_with_admin_users(
    request: TelegramWebhookAdminUsersBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookAdminUsersBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        admin_users,
        webhook_path,
        secret_token,
        dedup_config,
        tx,
    } = request;
    build_telegram_webhook_app_with_admin_users_and_command_rules(
        TelegramWebhookAdminRulesBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            admin_users,
            control_command_allow_from: None,
            control_command_rules: Vec::new(),
            webhook_path,
            secret_token,
            dedup_config,
            tx,
        },
    )
}

fn build_telegram_webhook_app_with_admin_users_and_command_rules(
    request: TelegramWebhookAdminRulesBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookAdminRulesBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        webhook_path,
        secret_token,
        dedup_config,
        tx,
    } = request;
    build_telegram_webhook_app_with_control_command_policy(
        TelegramWebhookControlPolicyBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy: TelegramControlCommandPolicy::new(
                admin_users,
                control_command_allow_from,
                control_command_rules,
            ),
            webhook_path,
            secret_token,
            dedup_config,
            tx,
        },
    )
}

/// Build a Telegram webhook app with structured control-command policy.
///
/// # Errors
/// Returns an error when channel construction or webhook app assembly fails.
pub fn build_telegram_webhook_app_with_control_command_policy(
    request: TelegramWebhookControlPolicyBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookControlPolicyBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy,
        webhook_path,
        secret_token,
        dedup_config,
        tx,
    } = request;
    build_telegram_webhook_app_with_partition_and_control_command_policy(
        TelegramWebhookPartitionPolicyBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy,
            webhook_path,
            secret_token,
            dedup_config,
            session_partition: TelegramSessionPartition::from_env(),
            tx,
        },
    )
}

/// Build a Telegram webhook app with explicit session partition strategy.
///
/// # Errors
/// Returns an error when channel construction or webhook app assembly fails.
pub fn build_telegram_webhook_app_with_partition(
    request: TelegramWebhookPartitionBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookPartitionBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        admin_users,
        webhook_path,
        secret_token,
        dedup_config,
        session_partition,
        tx,
    } = request;
    build_telegram_webhook_app_with_partition_and_control_command_policy(
        TelegramWebhookPartitionPolicyBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy: TelegramControlCommandPolicy::new(
                admin_users,
                None,
                Vec::new(),
            ),
            webhook_path,
            secret_token,
            dedup_config,
            session_partition,
            tx,
        },
    )
}

fn build_telegram_webhook_app_with_partition_and_control_command_policy(
    request: TelegramWebhookPartitionPolicyBuildRequest,
) -> Result<TelegramWebhookApp> {
    let TelegramWebhookPartitionPolicyBuildRequest {
        bot_token,
        allowed_users,
        allowed_groups,
        control_command_policy,
        webhook_path,
        secret_token,
        dedup_config,
        session_partition,
        tx,
    } = request;
    core::build_telegram_webhook_app_with_partition_and_control_command_policy(
        TelegramWebhookCoreBuildRequest {
            bot_token,
            allowed_users,
            allowed_groups,
            control_command_policy,
            webhook_path,
            secret_token,
            dedup_config,
            session_partition,
            tx,
        },
    )
}
