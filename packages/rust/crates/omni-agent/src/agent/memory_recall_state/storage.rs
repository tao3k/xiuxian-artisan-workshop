use crate::session::ChatMessage;

use super::types::{SessionMemoryRecallSnapshot, StoredSessionMemoryRecallSnapshot};

const MEMORY_RECALL_SNAPSHOT_SESSION_PREFIX: &str = "__session_memory_recall__:";
pub(crate) const MEMORY_RECALL_SNAPSHOT_MESSAGE_NAME: &str = "agent.memory.recall.snapshot";

pub(crate) fn snapshot_session_id(session_id: &str) -> String {
    format!("{MEMORY_RECALL_SNAPSHOT_SESSION_PREFIX}{session_id}")
}

pub(super) fn snapshot_chat_message(snapshot: &SessionMemoryRecallSnapshot) -> Option<ChatMessage> {
    let payload =
        serde_json::to_string(&StoredSessionMemoryRecallSnapshot::from(*snapshot)).ok()?;
    Some(ChatMessage {
        role: "system".to_string(),
        content: Some(payload),
        tool_calls: None,
        tool_call_id: None,
        name: Some(MEMORY_RECALL_SNAPSHOT_MESSAGE_NAME.to_string()),
    })
}

pub(super) fn parse_snapshot_chat_message(
    message: &ChatMessage,
) -> Option<SessionMemoryRecallSnapshot> {
    if let Some(name) = message.name.as_deref()
        && name != MEMORY_RECALL_SNAPSHOT_MESSAGE_NAME
    {
        return None;
    }
    let payload = message.content.as_deref()?;
    let stored: StoredSessionMemoryRecallSnapshot = serde_json::from_str(payload).ok()?;
    Some(stored.into_runtime())
}
