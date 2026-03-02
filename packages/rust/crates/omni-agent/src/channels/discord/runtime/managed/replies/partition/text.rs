use crate::channels::managed_runtime::session_partition::{
    SessionPartitionProfile, quick_toggle_usage, set_mode_usage, supported_modes_csv,
};

pub(in super::super::super) fn format_session_partition_status(current_mode: &str) -> String {
    let profile = SessionPartitionProfile::Discord;
    [
        "Session partition status.".to_string(),
        format!("current_mode={current_mode}"),
        format!("supported_modes={}", supported_modes_csv(profile)),
        format!("quick_toggle={}", quick_toggle_usage()),
        format!("set_mode={}", set_mode_usage(profile)),
        "scope=channel (takes effect for new incoming messages)".to_string(),
    ]
    .join("\n")
}

pub(in super::super::super) fn format_session_partition_updated(
    requested_mode: &str,
    current_mode: &str,
) -> String {
    [
        "Session partition updated.".to_string(),
        format!("requested_mode={requested_mode}"),
        format!("current_mode={current_mode}"),
        "scope=channel (takes effect for new incoming messages)".to_string(),
    ]
    .join("\n")
}

pub(in super::super::super) fn format_session_partition_admin_required(
    sender: &str,
    current_mode: &str,
) -> String {
    [
        "## Session Partition Permission Denied".to_string(),
        "- `reason`: `admin_required`".to_string(),
        format!("- `sender`: `{sender}`"),
        format!("- `current_mode`: `{current_mode}`"),
        "- `hint`: Ask an identity allowed by `discord.acl.control.allow_from` (or matching `discord.acl.control.rules` / `discord.acl.admin`) to run `/session partition ...` (or `/session scope ...`)."
            .to_string(),
    ]
    .join("\n")
}
