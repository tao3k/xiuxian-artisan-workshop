use super::input_normalization::normalize_command_input;
use super::types::ManagedSlashCommand;

/// Detect managed non-privileged slash commands that are ACL-scoped by
/// `Channel::is_authorized_for_slash_command`.
pub(crate) fn detect_managed_slash_command(input: &str) -> Option<ManagedSlashCommand> {
    let normalized = normalize_command_input(input);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let command = *tokens.first()?;

    if is_session_scope(command) {
        return detect_session_family_slash_command(&tokens);
    }
    if command.eq_ignore_ascii_case("feedback") {
        return detect_short_feedback_command(&tokens);
    }
    if command.eq_ignore_ascii_case("job") {
        return detect_job_status_command(&tokens);
    }
    if command.eq_ignore_ascii_case("jobs") {
        return detect_jobs_summary_command(&tokens);
    }
    if command.eq_ignore_ascii_case("bg") || command.eq_ignore_ascii_case("research") {
        return detect_background_submit_command(&tokens);
    }
    None
}

fn is_session_scope(command: &str) -> bool {
    command.eq_ignore_ascii_case("session")
        || command.eq_ignore_ascii_case("window")
        || command.eq_ignore_ascii_case("context")
}

fn is_json_token(token: &str) -> bool {
    token.eq_ignore_ascii_case("json")
}

fn is_status_alias(token: &str) -> bool {
    token.eq_ignore_ascii_case("status")
        || token.eq_ignore_ascii_case("stats")
        || token.eq_ignore_ascii_case("info")
}

fn is_memory_alias(token: &str) -> bool {
    token.eq_ignore_ascii_case("memory") || token.eq_ignore_ascii_case("recall")
}

fn is_feedback_direction(token: &str) -> bool {
    token.eq_ignore_ascii_case("up")
        || token.eq_ignore_ascii_case("success")
        || token.eq_ignore_ascii_case("positive")
        || token.eq_ignore_ascii_case("good")
        || token == "+"
        || token.eq_ignore_ascii_case("down")
        || token.eq_ignore_ascii_case("failure")
        || token.eq_ignore_ascii_case("negative")
        || token.eq_ignore_ascii_case("bad")
        || token.eq_ignore_ascii_case("fail")
        || token == "-"
}

fn detect_session_family_slash_command(tokens: &[&str]) -> Option<ManagedSlashCommand> {
    match tokens {
        [_] => Some(ManagedSlashCommand::SessionStatus),
        [_, one] if is_json_token(one) || is_status_alias(one) => {
            Some(ManagedSlashCommand::SessionStatus)
        }
        [_, one] if one.eq_ignore_ascii_case("budget") => Some(ManagedSlashCommand::SessionBudget),
        [_, one] if is_memory_alias(one) => Some(ManagedSlashCommand::SessionMemory),
        [_, one, two]
            if (is_status_alias(one)
                || one.eq_ignore_ascii_case("budget")
                || is_memory_alias(one))
                && is_json_token(two) =>
        {
            if is_status_alias(one) {
                Some(ManagedSlashCommand::SessionStatus)
            } else if one.eq_ignore_ascii_case("budget") {
                Some(ManagedSlashCommand::SessionBudget)
            } else {
                Some(ManagedSlashCommand::SessionMemory)
            }
        }
        [_, sub, direction]
            if sub.eq_ignore_ascii_case("feedback") && is_feedback_direction(direction) =>
        {
            Some(ManagedSlashCommand::SessionFeedback)
        }
        [_, sub, direction, fmt]
            if sub.eq_ignore_ascii_case("feedback")
                && is_feedback_direction(direction)
                && is_json_token(fmt) =>
        {
            Some(ManagedSlashCommand::SessionFeedback)
        }
        _ => None,
    }
}

fn detect_short_feedback_command(tokens: &[&str]) -> Option<ManagedSlashCommand> {
    match tokens {
        [_, direction] if is_feedback_direction(direction) => {
            Some(ManagedSlashCommand::SessionFeedback)
        }
        [_, direction, fmt] if is_feedback_direction(direction) && is_json_token(fmt) => {
            Some(ManagedSlashCommand::SessionFeedback)
        }
        _ => None,
    }
}

fn detect_job_status_command(tokens: &[&str]) -> Option<ManagedSlashCommand> {
    match tokens {
        [_, _job_id] => Some(ManagedSlashCommand::JobStatus),
        [_, _job_id, fmt] if is_json_token(fmt) => Some(ManagedSlashCommand::JobStatus),
        _ => None,
    }
}

fn detect_jobs_summary_command(tokens: &[&str]) -> Option<ManagedSlashCommand> {
    match tokens {
        [_] => Some(ManagedSlashCommand::JobsSummary),
        [_, fmt] if is_json_token(fmt) => Some(ManagedSlashCommand::JobsSummary),
        _ => None,
    }
}

fn detect_background_submit_command(tokens: &[&str]) -> Option<ManagedSlashCommand> {
    if tokens.len() >= 2 {
        Some(ManagedSlashCommand::BackgroundSubmit)
    } else {
        None
    }
}
