use serde_json::json;

use crate::channels::managed_runtime::session_partition::{
    SessionPartitionProfile, quick_toggle_usage, supported_modes,
};

pub(in super::super::super) fn format_session_partition_status_json(current_mode: &str) -> String {
    let profile = SessionPartitionProfile::Discord;
    json!({
        "kind": "session_partition",
        "updated": false,
        "current_mode": current_mode,
        "supported_modes": supported_modes(profile),
        "quick_toggle": quick_toggle_usage(),
        "scope": "channel",
    })
    .to_string()
}

pub(in super::super::super) fn format_session_partition_updated_json(
    requested_mode: &str,
    current_mode: &str,
) -> String {
    json!({
        "kind": "session_partition",
        "updated": true,
        "requested_mode": requested_mode,
        "current_mode": current_mode,
        "scope": "channel",
    })
    .to_string()
}

pub(in super::super::super) fn format_session_partition_error_json(
    requested_mode: &str,
    error: &str,
) -> String {
    json!({
        "kind": "session_partition",
        "updated": false,
        "requested_mode": requested_mode,
        "error": error,
    })
    .to_string()
}

pub(in super::super::super) fn format_session_partition_admin_required_json(
    sender: &str,
    current_mode: &str,
) -> String {
    json!({
        "kind": "session_partition",
        "updated": false,
        "reason": "admin_required",
        "sender": sender,
        "current_mode": current_mode,
        "hint": "Ask an identity allowed by discord.acl.control.allow_from (or discord.acl.control.rules / discord.acl.admin) to run /session partition ... (or /session scope ...).",
    })
    .to_string()
}
