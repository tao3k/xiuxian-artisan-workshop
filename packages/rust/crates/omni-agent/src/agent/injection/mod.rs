use crate::session::ChatMessage;
use xiuxian_qianhuan::{
    InjectionPolicy, InjectionSnapshot, PromptContextBlock, PromptContextCategory,
    PromptContextSource,
};

const TOOL_PAYLOAD_TRUNCATION_SUFFIX: &str = "...";

pub(super) struct InjectionNormalizationResult {
    pub(super) snapshot: Option<xiuxian_qianhuan::InjectionSnapshot>,
    pub(super) messages: Vec<ChatMessage>,
}

/// Pure semantic normalization of messages.
pub(super) fn normalize_messages_with_snapshot(
    session_id: &str,
    turn_id: u64,
    messages: Vec<ChatMessage>,
    policy: InjectionPolicy,
) -> InjectionNormalizationResult {
    let mut blocks = messages
        .iter()
        .enumerate()
        .filter_map(|(idx, message)| message_to_block(session_id, idx, message))
        .collect::<Vec<_>>();
    let (dropped_block_ids, truncated_block_ids) = apply_policy_budget(&mut blocks, &policy);

    let mut snapshot = InjectionSnapshot::from_blocks(
        format!("{session_id}:{turn_id}"),
        session_id.to_string(),
        turn_id,
        policy,
        None,
        blocks,
    );
    snapshot.dropped_block_ids = dropped_block_ids;
    snapshot.truncated_block_ids = truncated_block_ids;

    InjectionNormalizationResult {
        snapshot: Some(snapshot),
        messages,
    }
}

/// Truncate tool payload text according to injection policy `max_chars`.
#[must_use]
pub(super) fn truncate_tool_payload_for_policy(payload: &str, policy: &InjectionPolicy) -> String {
    let max_chars = policy.max_chars.max(1);
    if payload.chars().count() <= max_chars {
        return payload.to_string();
    }
    let keep = max_chars.saturating_sub(TOOL_PAYLOAD_TRUNCATION_SUFFIX.chars().count());
    let mut truncated = String::new();
    for ch in payload.chars().take(keep) {
        truncated.push(ch);
    }
    truncated.push_str(TOOL_PAYLOAD_TRUNCATION_SUFFIX);
    truncated
}

fn message_to_block(
    session_id: &str,
    index: usize,
    message: &ChatMessage,
) -> Option<PromptContextBlock> {
    let payload = render_message_payload(message)?;
    let (source, category, priority, anchor) = match message.role.as_str() {
        "system" => (
            PromptContextSource::Policy,
            PromptContextCategory::Policy,
            100,
            true,
        ),
        "user" => (
            PromptContextSource::RuntimeHint,
            PromptContextCategory::RuntimeHint,
            60,
            false,
        ),
        "assistant" => (
            PromptContextSource::Reflection,
            PromptContextCategory::Reflection,
            55,
            false,
        ),
        "tool" => (
            PromptContextSource::Knowledge,
            PromptContextCategory::Knowledge,
            40,
            false,
        ),
        _ => (
            PromptContextSource::RuntimeHint,
            PromptContextCategory::RuntimeHint,
            30,
            false,
        ),
    };

    Some(PromptContextBlock::new(
        format!("msg-{index}-{}", message.role),
        source,
        category,
        priority,
        session_id,
        payload,
        anchor,
    ))
}

fn render_message_payload(message: &ChatMessage) -> Option<String> {
    if let Some(content) = message.content.as_deref()
        && !content.trim().is_empty()
    {
        return Some(content.to_string());
    }

    let tool_calls = message.tool_calls.as_ref()?;
    if tool_calls.is_empty() {
        return None;
    }

    let summary = tool_calls
        .iter()
        .map(|call| call.function.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("[tool_calls] {summary}"))
}

fn apply_policy_budget(
    blocks: &mut Vec<PromptContextBlock>,
    policy: &InjectionPolicy,
) -> (Vec<String>, Vec<String>) {
    let mut dropped_block_ids = Vec::new();
    let mut truncated_block_ids = Vec::new();

    while blocks.len() > policy.max_blocks {
        let drop_index = blocks.iter().position(|block| !block.anchor).unwrap_or(0);
        dropped_block_ids.push(blocks[drop_index].block_id.clone());
        blocks.remove(drop_index);
    }

    while blocks
        .iter()
        .map(|block| block.payload_chars)
        .sum::<usize>()
        > policy.max_chars
    {
        let total_chars = blocks
            .iter()
            .map(|block| block.payload_chars)
            .sum::<usize>();
        let overflow = total_chars.saturating_sub(policy.max_chars);
        if overflow == 0 {
            break;
        }

        if let Some(drop_index) = blocks.iter().position(|block| !block.anchor) {
            dropped_block_ids.push(blocks[drop_index].block_id.clone());
            blocks.remove(drop_index);
            continue;
        }

        let Some(last_index) = blocks.len().checked_sub(1) else {
            break;
        };
        let block = &mut blocks[last_index];
        if block.payload_chars <= TOOL_PAYLOAD_TRUNCATION_SUFFIX.chars().count() {
            break;
        }

        let keep_chars = block
            .payload_chars
            .saturating_sub(overflow)
            .saturating_sub(TOOL_PAYLOAD_TRUNCATION_SUFFIX.chars().count())
            .max(1);
        let mut truncated = block.payload.chars().take(keep_chars).collect::<String>();
        truncated.push_str(TOOL_PAYLOAD_TRUNCATION_SUFFIX);
        block.payload = truncated;
        block.payload_chars = block.payload.chars().count();
        truncated_block_ids.push(block.block_id.clone());
    }

    (dropped_block_ids, truncated_block_ids)
}
