use omni_tokenizer::count_tokens;

use crate::session::ChatMessage;

use super::types::{
    ClassifiedMessages, ContextBudgetReport, IndexedMessage, MessageClass,
    SESSION_SUMMARY_MESSAGE_NAME,
};

pub(super) fn classify_messages_for_budget(
    messages: Vec<ChatMessage>,
    report: &mut ContextBudgetReport,
) -> ClassifiedMessages {
    let mut regular_system = Vec::new();
    let mut summary_system = Vec::new();
    let mut non_system = Vec::new();

    for (index, message) in messages.into_iter().enumerate() {
        let class = classify_message(&message);
        let original_tokens = estimated_message_tokens(&message);
        report.class_mut(class).record_input(original_tokens);
        report.pre_messages = report.pre_messages.saturating_add(1);
        report.pre_tokens = report.pre_tokens.saturating_add(original_tokens);

        let indexed = IndexedMessage {
            index,
            class,
            original_tokens,
            message,
        };
        match class {
            MessageClass::Non => non_system.push(indexed),
            MessageClass::Regular => regular_system.push(indexed),
            MessageClass::Summary => summary_system.push(indexed),
        }
    }

    ClassifiedMessages {
        regular: regular_system,
        summary: summary_system,
        non: non_system,
    }
}

pub(super) fn estimated_message_tokens(message: &ChatMessage) -> usize {
    let mut total = estimated_message_overhead_tokens(message);
    if let Some(content) = &message.content {
        total = total.saturating_add(count_tokens(content));
    }
    total
}

pub(super) fn estimated_message_overhead_tokens(message: &ChatMessage) -> usize {
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

fn classify_message(message: &ChatMessage) -> MessageClass {
    if message.role == "system" {
        if is_summary_system_message(message) {
            MessageClass::Summary
        } else {
            MessageClass::Regular
        }
    } else {
        MessageClass::Non
    }
}

fn is_summary_system_message(message: &ChatMessage) -> bool {
    message.role == "system" && message.name.as_deref() == Some(SESSION_SUMMARY_MESSAGE_NAME)
}
