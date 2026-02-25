use xiuxian_qianhuan::PromptContextBlock;

pub(super) fn apply_char_budget(
    blocks: Vec<PromptContextBlock>,
    max_chars: usize,
) -> (Vec<PromptContextBlock>, Vec<String>, Vec<String>) {
    if max_chars == 0 {
        let dropped = blocks.into_iter().map(|block| block.block_id).collect();
        return (Vec::new(), dropped, Vec::new());
    }

    let mut kept = Vec::new();
    let mut dropped_block_ids = Vec::new();
    let mut truncated_block_ids = Vec::new();
    let mut used_chars = 0usize;

    for mut block in blocks {
        if used_chars >= max_chars {
            dropped_block_ids.push(block.block_id);
            continue;
        }

        let remaining = max_chars.saturating_sub(used_chars);
        if block.payload_chars <= remaining {
            used_chars = used_chars.saturating_add(block.payload_chars);
            kept.push(block);
            continue;
        }

        if remaining == 0 {
            dropped_block_ids.push(block.block_id);
            continue;
        }

        block.payload = truncate_chars(&block.payload, remaining);
        block.payload_chars = block.payload.chars().count();
        used_chars = used_chars.saturating_add(block.payload_chars);
        truncated_block_ids.push(block.block_id.clone());
        kept.push(block);
    }

    (kept, dropped_block_ids, truncated_block_ids)
}

pub(super) fn prioritize_anchors(blocks: Vec<PromptContextBlock>) -> Vec<PromptContextBlock> {
    let (anchors, others): (Vec<_>, Vec<_>) = blocks.into_iter().partition(|block| block.anchor);
    anchors.into_iter().chain(others).collect()
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let mut out = input
        .chars()
        .take(max_chars.saturating_sub(3))
        .collect::<String>();
    out.push_str("...");
    out
}
