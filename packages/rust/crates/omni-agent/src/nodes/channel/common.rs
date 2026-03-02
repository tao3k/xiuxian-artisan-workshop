pub(super) fn log_control_command_allow_override(provider: &str, entries: Option<&[String]>) {
    if let Some(entries) = entries {
        if entries.is_empty() {
            tracing::warn!(
                provider = %provider,
                "{provider}.control_command_allow_from is configured but empty; privileged control commands are denied for all senders"
            );
        } else {
            tracing::info!(
                provider = %provider,
                entries = entries.len(),
                "{provider}.control_command_allow_from override is active"
            );
        }
    }
}

pub(super) fn log_slash_command_allow_override(provider: &str, entries: Option<&[String]>) {
    if let Some(entries) = entries {
        if entries.is_empty() {
            tracing::warn!(
                provider = %provider,
                "{provider}.slash_command_allow_from is configured but empty; managed slash commands are denied for all non-admin senders"
            );
        } else {
            tracing::info!(
                provider = %provider,
                entries = entries.len(),
                "{provider}.slash_command_allow_from override is active"
            );
        }
    }
}
