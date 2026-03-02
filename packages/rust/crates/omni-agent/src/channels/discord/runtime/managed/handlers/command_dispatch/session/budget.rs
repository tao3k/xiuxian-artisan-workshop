use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::managed_commands::SLASH_SCOPE_SESSION_BUDGET;
use crate::channels::traits::{Channel, ChannelMessage};

use super::super::super::super::parsing::CommandOutputFormat;
use super::super::super::super::replies::{
    format_context_budget_not_found_json, format_context_budget_snapshot,
    format_context_budget_snapshot_json,
};
use super::super::super::auth::ensure_slash_command_authorized;
use super::super::super::events::{
    EVENT_DISCORD_COMMAND_SESSION_BUDGET_JSON_REPLIED, EVENT_DISCORD_COMMAND_SESSION_BUDGET_REPLIED,
};
use super::super::super::send::send_response;

pub(in super::super) async fn handle_session_budget(
    agent: &Arc<Agent>,
    channel: &Arc<dyn Channel>,
    msg: &ChannelMessage,
    session_id: &str,
    format: CommandOutputFormat,
) {
    if !ensure_slash_command_authorized(channel, msg, SLASH_SCOPE_SESSION_BUDGET, "/session budget")
        .await
    {
        return;
    }
    let command_event = if format.is_json() {
        EVENT_DISCORD_COMMAND_SESSION_BUDGET_JSON_REPLIED
    } else {
        EVENT_DISCORD_COMMAND_SESSION_BUDGET_REPLIED
    };
    let response = match agent.inspect_context_budget_snapshot(session_id).await {
        Some(snapshot) if format.is_json() => format_context_budget_snapshot_json(&snapshot),
        Some(snapshot) => format_context_budget_snapshot(&snapshot),
        None if format.is_json() => format_context_budget_not_found_json(),
        None => {
            "No context budget snapshot found for this session yet.\nRun at least one normal turn first (non-command message)."
                .to_string()
        }
    };
    send_response(channel, &msg.recipient, response, msg, command_event).await;
}
