use crate::channels::control_command_authorization::ControlCommandPolicy;
use crate::channels::telegram::build_telegram_acl_overrides_from_settings;
use crate::config::TelegramSettings;

use super::super::TelegramSlashCommandPolicy;
use super::super::group_policy::{
    TelegramGroupPolicyConfig, TelegramGroupPolicyMode, parse_group_policy_mode,
};
use super::group_overrides::parse_group_overrides;
use super::normalization::{
    normalize_allowed_group_entries, normalize_allowed_user_entries_with_context,
    normalize_control_command_policy, normalize_group_allow_from, normalize_slash_command_policy,
};
use super::parsing::{
    parse_optional_comma_entries, resolve_bool_env_or_setting, resolve_optional_env_or_setting,
    resolve_string_env_or_setting,
};
use super::slash_policy::build_slash_command_policy;
use super::types::{TELEGRAM_ACL_FIELD_ALLOWED_USERS, TelegramAclConfig};

pub(in crate::channels::telegram::channel) fn resolve_acl_config_from_settings(
    settings: TelegramSettings,
) -> anyhow::Result<TelegramAclConfig> {
    let acl_overrides = build_telegram_acl_overrides_from_settings(&settings)?;
    let session_admin_persist = resolve_bool_env_or_setting(
        "OMNI_AGENT_TELEGRAM_SESSION_ADMIN_PERSIST",
        settings.session_admin_persist,
        false,
    );
    let group_policy_raw = resolve_string_env_or_setting(
        "OMNI_AGENT_TELEGRAM_GROUP_POLICY",
        settings.group_policy,
        "open",
    );
    let group_allow_from_raw = resolve_optional_env_or_setting(
        "OMNI_AGENT_TELEGRAM_GROUP_ALLOW_FROM",
        settings.group_allow_from,
    );
    let require_mention = resolve_bool_env_or_setting(
        "OMNI_AGENT_TELEGRAM_REQUIRE_MENTION",
        settings.require_mention,
        false,
    );
    let group_entries = settings.groups.unwrap_or_default();
    let control_command_rules = acl_overrides.control_command_rules;

    let allowed_users = normalize_allowed_user_entries_with_context(
        acl_overrides.allowed_users,
        TELEGRAM_ACL_FIELD_ALLOWED_USERS,
    );
    let allowed_groups = normalize_allowed_group_entries(acl_overrides.allowed_groups);
    let group_policy = parse_group_policy_mode(group_policy_raw.as_str(), "telegram.group_policy")
        .unwrap_or(TelegramGroupPolicyMode::Open);
    let group_allow_from =
        normalize_group_allow_from(parse_optional_comma_entries(group_allow_from_raw));
    let admin_users = acl_overrides.admin_users;
    let control_command_allow_from = acl_overrides.control_command_allow_from;

    let slash_command_policy = TelegramSlashCommandPolicy {
        global: acl_overrides.slash_command_allow_from,
        session_status: acl_overrides.slash_session_status_allow_from,
        session_budget: acl_overrides.slash_session_budget_allow_from,
        session_memory: acl_overrides.slash_session_memory_allow_from,
        session_feedback: acl_overrides.slash_session_feedback_allow_from,
        job_status: acl_overrides.slash_job_allow_from,
        jobs_summary: acl_overrides.slash_jobs_allow_from,
        background_submit: acl_overrides.slash_bg_allow_from,
    };

    let control_command_policy = normalize_control_command_policy(ControlCommandPolicy::new(
        admin_users.clone(),
        control_command_allow_from,
        control_command_rules,
    ));
    let slash_command_policy = normalize_slash_command_policy(build_slash_command_policy(
        admin_users,
        slash_command_policy,
    ));
    let group_policy_config = TelegramGroupPolicyConfig {
        group_policy,
        group_allow_from,
        require_mention,
        groups: parse_group_overrides(group_entries),
    };

    Ok(TelegramAclConfig {
        allowed_users,
        allowed_groups,
        control_command_policy,
        slash_command_policy,
        group_policy_config,
        session_admin_persist,
    })
}
