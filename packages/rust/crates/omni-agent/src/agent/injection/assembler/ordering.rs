use xiuxian_qianhuan::{
    InjectionOrderStrategy, InjectionPolicy, PromptContextBlock, PromptContextCategory,
};

pub(super) fn sort_blocks(blocks: &mut [PromptContextBlock], policy: &InjectionPolicy) {
    match policy.ordering {
        InjectionOrderStrategy::PriorityDesc => {
            blocks.sort_by(|left, right| {
                right
                    .priority
                    .cmp(&left.priority)
                    .then_with(|| left.block_id.cmp(&right.block_id))
            });
        }
        InjectionOrderStrategy::CategoryThenPriority => {
            blocks.sort_by(|left, right| {
                category_rank(&policy.enabled_categories, left.category)
                    .cmp(&category_rank(&policy.enabled_categories, right.category))
                    .then_with(|| right.priority.cmp(&left.priority))
                    .then_with(|| left.block_id.cmp(&right.block_id))
            });
        }
    }
}

fn category_rank(enabled: &[PromptContextCategory], category: PromptContextCategory) -> usize {
    enabled
        .iter()
        .position(|value| *value == category)
        .unwrap_or(usize::MAX)
}
