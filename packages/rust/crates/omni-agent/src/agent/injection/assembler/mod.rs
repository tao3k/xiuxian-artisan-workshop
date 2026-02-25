use xiuxian_qianhuan::{InjectionPolicy, InjectionSnapshot, PromptContextBlock};

mod budget;
mod ordering;
mod role_mix;
mod util;

use budget::{apply_char_budget, prioritize_anchors};
use ordering::sort_blocks;
use role_mix::select_role_mix;
use util::dedup_preserve_order;

pub(super) fn assemble_snapshot(
    session_id: &str,
    turn_id: u64,
    policy: InjectionPolicy,
    blocks: Vec<PromptContextBlock>,
) -> InjectionSnapshot {
    let mut dropped_block_ids = Vec::new();

    let mut selected = blocks
        .into_iter()
        .filter_map(|mut block| {
            if !policy.enabled_categories.contains(&block.category) {
                dropped_block_ids.push(block.block_id);
                return None;
            }
            block.anchor = block.anchor || policy.anchor_categories.contains(&block.category);
            Some(block)
        })
        .collect::<Vec<_>>();

    sort_blocks(&mut selected, &policy);
    let role_mix = Some(select_role_mix(&policy, &selected));

    let mut retained = Vec::new();
    for block in selected {
        if retained.len() < policy.max_blocks {
            retained.push(block);
            continue;
        }

        if block.anchor
            && let Some(replace_index) = retained.iter().rposition(|existing| !existing.anchor)
        {
            let evicted = std::mem::replace(&mut retained[replace_index], block);
            dropped_block_ids.push(evicted.block_id);
            continue;
        }

        dropped_block_ids.push(block.block_id);
    }

    let (final_blocks, mut budget_dropped, truncated_block_ids) =
        apply_char_budget(prioritize_anchors(retained), policy.max_chars);
    dropped_block_ids.append(&mut budget_dropped);

    let mut snapshot = InjectionSnapshot::from_blocks(
        format!("injection:{session_id}:{turn_id}"),
        session_id,
        turn_id,
        policy,
        role_mix,
        final_blocks,
    );
    snapshot.dropped_block_ids = dedup_preserve_order(dropped_block_ids);
    snapshot.truncated_block_ids = dedup_preserve_order(truncated_block_ids);
    snapshot
}
