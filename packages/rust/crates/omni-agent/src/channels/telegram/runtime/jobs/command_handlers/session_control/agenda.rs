use std::sync::Arc;

use crate::agent::Agent;
use crate::channels::telegram::commands::is_agenda_command;
use crate::channels::traits::{Channel, ChannelMessage};

use super::super::super::observability::send_with_observability;
use super::EVENT_TELEGRAM_COMMAND_AGENDA_REPLIED;

pub(in crate::channels::telegram::runtime::jobs) async fn try_handle_agenda_command(
    msg: &ChannelMessage,
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
) -> bool {
    if !is_agenda_command(&msg.content) {
        return false;
    }

    let response = match agent.get_heyi() {
        Some(heyi) => {
            match heyi.sync_from_disk() {
                Ok(summary) => {
                    tracing::info!(
                        event = "telegram.zhixing.sync.completed",
                        journal_documents = summary.journal_documents,
                        agenda_documents = summary.agenda_documents,
                        task_entities = summary.task_entities,
                        entities_added = summary.entities_added,
                        relations_linked = summary.relations_linked,
                        "Zhixing-Heyi library-level sync succeeded"
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        event = "telegram.zhixing.sync.failed",
                        error = %error,
                        "Zhixing-Heyi library-level sync failed"
                    );
                }
            }

            match heyi.render_agenda() {
                Ok(rendered) => rendered,
                Err(error) => format!("Failed to render agenda: {error}"),
            }
        }
        None => "Zhixing agenda service is unavailable in this runtime.".to_string(),
    };

    send_with_observability(
        channel,
        &response,
        &msg.recipient,
        "Failed to send agenda response",
        Some(EVENT_TELEGRAM_COMMAND_AGENDA_REPLIED),
        Some(&msg.session_key),
    )
    .await;
    true
}
