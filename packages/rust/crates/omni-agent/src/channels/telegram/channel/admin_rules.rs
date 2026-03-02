use crate::channels::control_command_rule_specs::{
    CommandSelectorAuthRule, parse_control_command_rule,
};
use anyhow::Result;

use super::identity::normalize_user_identity;

/// Parsed Telegram command-admin rule for control-command authorization.
pub type TelegramCommandAdminRule = CommandSelectorAuthRule;

/// Build one Telegram command-admin rule from selectors and allowed users.
///
/// # Errors
/// Returns an error when selectors or users are invalid.
pub fn build_telegram_command_admin_rule(
    selectors: Vec<String>,
    allowed_users: Vec<String>,
) -> Result<TelegramCommandAdminRule> {
    parse_control_command_rule(
        selectors,
        allowed_users,
        "admin command rule",
        normalize_user_identity,
    )
}
