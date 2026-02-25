mod read_ops;
mod write_ops;

use omni_window::TurnSlot;

use super::super::message::ChatMessage;

fn turn_slots_to_messages(slots: &[TurnSlot]) -> Vec<ChatMessage> {
    slots
        .iter()
        .map(|slot| ChatMessage {
            role: slot.role.clone(),
            content: Some(slot.content.clone()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        })
        .collect()
}
