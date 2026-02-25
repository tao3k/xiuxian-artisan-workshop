use omni_tokenizer::count_tokens;

use crate::session::ChatMessage;

/// Estimate total token footprint for a message list.
pub(crate) fn estimate_messages_tokens(messages: &[ChatMessage]) -> usize {
    messages.iter().map(estimated_message_tokens).sum()
}

fn estimated_message_tokens(message: &ChatMessage) -> usize {
    let mut total = estimated_message_overhead_tokens(message);
    if let Some(content) = &message.content {
        total = total.saturating_add(count_tokens(content));
    }
    total
}

fn estimated_message_overhead_tokens(message: &ChatMessage) -> usize {
    let mut total = 6usize.saturating_add(count_tokens(&message.role));
    if let Some(name) = &message.name {
        total = total.saturating_add(count_tokens(name));
    }
    if let Some(tool_call_id) = &message.tool_call_id {
        total = total.saturating_add(count_tokens(tool_call_id));
    }
    if let Some(tool_calls) = &message.tool_calls {
        let encoded = serde_json::to_string(tool_calls).unwrap_or_default();
        total = total.saturating_add(count_tokens(&encoded));
    }
    total
}
