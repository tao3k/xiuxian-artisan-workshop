use omni_tokenizer::truncate;

use crate::session::ChatMessage;

use super::classify::{estimated_message_overhead_tokens, estimated_message_tokens};

pub(super) fn truncate_message_to_budget(
    message: ChatMessage,
    budget_tokens: usize,
) -> Option<ChatMessage> {
    if budget_tokens == 0 {
        return None;
    }
    let current = estimated_message_tokens(&message);
    if current <= budget_tokens {
        return Some(message);
    }

    let content = message.content.clone()?;

    let overhead = estimated_message_overhead_tokens(&message);
    if overhead >= budget_tokens {
        return None;
    }
    let content_budget = budget_tokens.saturating_sub(overhead).max(1);
    let truncated_content = truncate(&content, content_budget);
    if truncated_content.trim().is_empty() {
        return None;
    }
    let mut trimmed = message;
    trimmed.content = Some(truncated_content);
    Some(trimmed)
}
