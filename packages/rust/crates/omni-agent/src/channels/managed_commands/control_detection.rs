use crate::channels::managed_runtime::parsing::{
    parse_session_partition_command as parse_session_partition_shared,
    parse_session_partition_mode_token,
};

use super::input_normalization::normalize_command_input;
use super::types::ManagedControlCommand;

/// Detect privileged managed control commands that are ACL-scoped by
/// `Channel::is_authorized_for_control_command`.
pub(crate) fn detect_managed_control_command(input: &str) -> Option<ManagedControlCommand> {
    let normalized = normalize_command_input(input);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let command = *tokens.first()?;

    if (command.eq_ignore_ascii_case("reset") || command.eq_ignore_ascii_case("clear"))
        && tokens.len() == 1
    {
        return Some(ManagedControlCommand::Reset);
    }

    if command.eq_ignore_ascii_case("resume") {
        return match tokens.as_slice() {
            [_] => Some(ManagedControlCommand::ResumeRestore),
            [_, sub]
                if sub.eq_ignore_ascii_case("status")
                    || sub.eq_ignore_ascii_case("stats")
                    || sub.eq_ignore_ascii_case("info") =>
            {
                Some(ManagedControlCommand::ResumeStatus)
            }
            [_, sub] if sub.eq_ignore_ascii_case("drop") || sub.eq_ignore_ascii_case("discard") => {
                Some(ManagedControlCommand::ResumeDrop)
            }
            _ => None,
        };
    }

    if is_session_partition_control_command(normalized) {
        return Some(ManagedControlCommand::SessionPartition);
    }
    if is_session_admin_control_command(normalized) {
        return Some(ManagedControlCommand::SessionAdmin);
    }
    if is_session_injection_control_command(normalized) {
        return Some(ManagedControlCommand::SessionInjection);
    }

    None
}

fn is_session_partition_control_command(input: &str) -> bool {
    parse_session_partition_shared(input, parse_session_partition_mode_token).is_some()
}

fn is_session_scope(scope: &str) -> bool {
    scope.eq_ignore_ascii_case("session")
        || scope.eq_ignore_ascii_case("window")
        || scope.eq_ignore_ascii_case("context")
}

fn is_session_admin_control_command(input: &str) -> bool {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    match tokens.as_slice() {
        [scope, admin] if is_session_scope(scope) && admin.eq_ignore_ascii_case("admin") => true,
        [scope, admin, third]
            if is_session_scope(scope)
                && admin.eq_ignore_ascii_case("admin")
                && (third.eq_ignore_ascii_case("json")
                    || third.eq_ignore_ascii_case("list")
                    || third.eq_ignore_ascii_case("clear")) =>
        {
            true
        }
        [scope, admin, action, ..]
            if is_session_scope(scope)
                && admin.eq_ignore_ascii_case("admin")
                && (action.eq_ignore_ascii_case("set")
                    || action.eq_ignore_ascii_case("add")
                    || action.eq_ignore_ascii_case("remove")
                    || action.eq_ignore_ascii_case("rm")
                    || action.eq_ignore_ascii_case("del")) =>
        {
            true
        }
        _ => false,
    }
}

fn is_session_injection_control_command(input: &str) -> bool {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    match tokens.as_slice() {
        [scope, inject]
            if is_session_scope(scope)
                && (inject.eq_ignore_ascii_case("inject")
                    || inject.eq_ignore_ascii_case("injection")) =>
        {
            true
        }
        [scope, inject, third]
            if is_session_scope(scope)
                && (inject.eq_ignore_ascii_case("inject")
                    || inject.eq_ignore_ascii_case("injection"))
                && (third.eq_ignore_ascii_case("json")
                    || third.eq_ignore_ascii_case("status")
                    || third.eq_ignore_ascii_case("clear")) =>
        {
            true
        }
        [scope, inject, ..]
            if is_session_scope(scope)
                && (inject.eq_ignore_ascii_case("inject")
                    || inject.eq_ignore_ascii_case("injection")) =>
        {
            true
        }
        _ => false,
    }
}
