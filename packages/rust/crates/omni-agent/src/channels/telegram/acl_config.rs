use crate::config::{
    RuntimeSettings, TelegramAclControlSettings, TelegramAclPrincipalSettings,
    TelegramAclSlashSettings, TelegramSettings,
};

use super::channel::{TelegramCommandAdminRule, build_telegram_command_admin_rule};

type TelegramSlashAclOverrides = (
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
    Option<Vec<String>>,
);

/// Runtime ACL override bundle resolved for Telegram channel execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TelegramAclOverrides {
    /// Resolved allowlist users.
    pub allowed_users: Vec<String>,
    /// Resolved allowlist groups.
    pub allowed_groups: Vec<String>,
    /// Resolved admin users.
    pub admin_users: Vec<String>,
    /// Optional global control-command allow identities.
    pub control_command_allow_from: Option<Vec<String>>,
    /// Parsed command-scoped control ACL rules.
    pub control_command_rules: Vec<TelegramCommandAdminRule>,
    /// Optional global slash-command allow identities.
    pub slash_command_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for session status.
    pub slash_session_status_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for session budget.
    pub slash_session_budget_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for session memory.
    pub slash_session_memory_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for session feedback.
    pub slash_session_feedback_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for job status.
    pub slash_job_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for jobs summary.
    pub slash_jobs_allow_from: Option<Vec<String>>,
    /// Optional slash allow identities for background submit.
    pub slash_bg_allow_from: Option<Vec<String>>,
}

/// Build telegram ACL runtime overrides from full runtime settings.
///
/// # Errors
/// Returns an error when control-rule command policy parsing fails.
pub fn build_telegram_acl_overrides(
    settings: &RuntimeSettings,
) -> anyhow::Result<TelegramAclOverrides> {
    build_telegram_acl_overrides_from_settings(&settings.telegram)
}

/// Build telegram ACL runtime overrides from telegram-specific settings.
///
/// # Errors
/// Returns an error when control-rule command policy parsing fails.
pub fn build_telegram_acl_overrides_from_settings(
    settings: &TelegramSettings,
) -> anyhow::Result<TelegramAclOverrides> {
    let acl = &settings.acl;

    let allowed_users = acl
        .allow
        .as_ref()
        .and_then(|allow| allow.users.as_ref())
        .map(|entries| normalize_entries(entries))
        .unwrap_or_default();
    let allowed_groups = acl
        .allow
        .as_ref()
        .and_then(|allow| allow.groups.as_ref())
        .map(|entries| normalize_entries(entries))
        .unwrap_or_default();
    let admin_users = acl
        .admin
        .as_ref()
        .and_then(collect_principals)
        .unwrap_or_default();
    let control_command_allow_from = acl
        .control
        .as_ref()
        .and_then(|control| control.allow_from.as_ref())
        .and_then(collect_principals);
    let control_command_rules = acl
        .control
        .as_ref()
        .map(control_rules)
        .transpose()?
        .unwrap_or_default();

    let (
        slash_command_allow_from,
        slash_session_status_allow_from,
        slash_session_budget_allow_from,
        slash_session_memory_allow_from,
        slash_session_feedback_allow_from,
        slash_job_status_allow_from,
        slash_jobs_summary_allow_from,
        slash_bg_allow_from,
    ) = slash_overrides(acl.slash.as_ref());

    Ok(TelegramAclOverrides {
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
        slash_job_allow_from: slash_job_status_allow_from,
        slash_jobs_allow_from: slash_jobs_summary_allow_from,
        slash_bg_allow_from,
    })
}

fn normalize_entries(entries: &[String]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
        .collect()
}

fn collect_principals(principal: &TelegramAclPrincipalSettings) -> Option<Vec<String>> {
    let users = principal.users.as_deref()?;
    Some(normalize_entries(users))
}

fn control_rules(
    control: &TelegramAclControlSettings,
) -> anyhow::Result<Vec<TelegramCommandAdminRule>> {
    let Some(rules) = control.rules.as_ref() else {
        return Ok(Vec::new());
    };
    let mut parsed_rules = Vec::new();
    for (index, rule) in rules.iter().enumerate() {
        let commands = normalize_entries(&rule.commands);
        if commands.is_empty() {
            tracing::warn!("telegram acl control rule ignored: empty commands");
            continue;
        }
        let Some(principals) = collect_principals(&rule.allow) else {
            tracing::warn!(
                commands = %commands.join(","),
                "telegram acl control rule ignored: no allow principals configured"
            );
            continue;
        };
        if principals.is_empty() {
            tracing::warn!(
                commands = %commands.join(","),
                "telegram acl control rule ignored: allow principals resolved to empty set"
            );
            continue;
        }
        let parsed_rule =
            build_telegram_command_admin_rule(commands, principals).map_err(|error| {
                anyhow::anyhow!("telegram.acl.control.rules[{index}].commands: {error}")
            })?;
        parsed_rules.push(parsed_rule);
    }
    Ok(parsed_rules)
}

fn slash_overrides(slash: Option<&TelegramAclSlashSettings>) -> TelegramSlashAclOverrides {
    let Some(slash) = slash else {
        return (None, None, None, None, None, None, None, None);
    };

    (
        slash.global.as_ref().and_then(collect_principals),
        slash.session_status.as_ref().and_then(collect_principals),
        slash.session_budget.as_ref().and_then(collect_principals),
        slash.session_memory.as_ref().and_then(collect_principals),
        slash.session_feedback.as_ref().and_then(collect_principals),
        slash.job_status.as_ref().and_then(collect_principals),
        slash.jobs_summary.as_ref().and_then(collect_principals),
        slash
            .background_submit
            .as_ref()
            .and_then(collect_principals),
    )
}
