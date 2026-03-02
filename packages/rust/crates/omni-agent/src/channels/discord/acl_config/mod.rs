use crate::config::{
    DiscordAclAllowSettings, DiscordAclControlSettings, DiscordAclPrincipalSettings,
    DiscordAclSettings, DiscordAclSlashSettings, RuntimeSettings,
};

use super::channel::{DiscordCommandAdminRule, build_discord_command_admin_rule};
use control_rules::control_rules;
use principals::{collect_principals, guilds_list_from_allow, principal_list_from_allow};
use role_aliases::normalize_role_aliases;
use slash::slash_overrides;

mod control_rules;
mod principals;
mod role_aliases;
mod slash;

/// Runtime ACL override bundle resolved for Discord channel execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiscordAclOverrides {
    /// Resolved allowlist users/roles.
    pub allowed_users: Vec<String>,
    /// Resolved allowlist guild ids.
    pub allowed_guilds: Vec<String>,
    /// Resolved admin identities.
    pub admin_users: Option<Vec<String>>,
    /// Optional global control-command allow identities.
    pub control_command_allow_from: Option<Vec<String>>,
    /// Parsed command-scoped control ACL rules.
    pub control_command_rules: Vec<DiscordCommandAdminRule>,
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

/// Build Discord runtime ACL overrides from settings.
///
/// # Errors
/// Returns an error when ACL command-rule parsing fails.
pub fn build_discord_acl_overrides(
    settings: &RuntimeSettings,
) -> anyhow::Result<DiscordAclOverrides> {
    let acl = &settings.discord.acl;
    let role_aliases = normalize_role_aliases(acl);

    let allowed_users = acl
        .allow
        .as_ref()
        .and_then(|allow| principal_list_from_allow(allow, &role_aliases))
        .unwrap_or_default();
    let allowed_guilds = acl
        .allow
        .as_ref()
        .and_then(guilds_list_from_allow)
        .unwrap_or_default();
    let admin_users = acl
        .admin
        .as_ref()
        .and_then(|principal| collect_principals(principal, &role_aliases));
    let control_command_allow_from = acl
        .control
        .as_ref()
        .and_then(|control| control.allow_from.as_ref())
        .and_then(|allow_from| collect_principals(allow_from, &role_aliases));
    let control_command_rules = acl
        .control
        .as_ref()
        .map(|control| control_rules(control, &role_aliases))
        .transpose()?
        .unwrap_or_default();

    let slash_overrides = slash_overrides(acl.slash.as_ref(), &role_aliases);

    Ok(DiscordAclOverrides {
        allowed_users,
        allowed_guilds,
        admin_users,
        control_command_allow_from,
        control_command_rules,
        slash_command_allow_from: slash_overrides.command,
        slash_session_status_allow_from: slash_overrides.session_status,
        slash_session_budget_allow_from: slash_overrides.session_budget,
        slash_session_memory_allow_from: slash_overrides.session_memory,
        slash_session_feedback_allow_from: slash_overrides.session_feedback,
        slash_job_allow_from: slash_overrides.job_status,
        slash_jobs_allow_from: slash_overrides.jobs_summary,
        slash_bg_allow_from: slash_overrides.background_submit,
    })
}
