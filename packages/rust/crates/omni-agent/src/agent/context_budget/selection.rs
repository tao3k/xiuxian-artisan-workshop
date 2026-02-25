use crate::config::ContextBudgetStrategy;
use crate::session::ChatMessage;

use super::classify::estimated_message_tokens;
use super::truncate::truncate_message_to_budget;
use super::types::{ClassifiedMessages, ContextBudgetReport, IndexedMessage, SelectedMessage};

pub(super) fn select_latest_non_system(
    non_system: &[IndexedMessage],
    effective_budget: usize,
) -> Option<SelectedMessage> {
    let latest_non_system = non_system.last()?.clone();
    let trimmed = truncate_message_to_budget(latest_non_system.message, effective_budget)?;
    let kept_tokens = estimated_message_tokens(&trimmed);
    Some(SelectedMessage {
        index: latest_non_system.index,
        class: latest_non_system.class,
        original_tokens: latest_non_system.original_tokens,
        kept_tokens,
        message: trimmed,
    })
}

pub(super) fn build_budget_candidates(
    classified: ClassifiedMessages,
    strategy: ContextBudgetStrategy,
) -> Vec<IndexedMessage> {
    let ClassifiedMessages {
        regular,
        summary,
        non,
    } = classified;
    let mut candidates = Vec::new();
    candidates.extend(regular);
    match strategy {
        ContextBudgetStrategy::RecentFirst => {
            if !non.is_empty() {
                candidates.extend(non[..non.len().saturating_sub(1)].iter().rev().cloned());
            }
            candidates.extend(summary.into_iter().rev());
        }
        ContextBudgetStrategy::SummaryFirst => {
            candidates.extend(summary.into_iter().rev());
            if !non.is_empty() {
                candidates.extend(non[..non.len().saturating_sub(1)].iter().rev().cloned());
            }
        }
    }
    candidates
}

pub(super) fn select_candidate_messages(
    candidates: Vec<IndexedMessage>,
    effective_budget: usize,
    used_tokens: &mut usize,
    selected: &mut Vec<SelectedMessage>,
) {
    for candidate in candidates {
        if *used_tokens >= effective_budget {
            break;
        }
        let remaining = effective_budget.saturating_sub(*used_tokens);
        if let Some(trimmed) = truncate_message_to_budget(candidate.message, remaining) {
            let kept_tokens = estimated_message_tokens(&trimmed);
            *used_tokens = (*used_tokens).saturating_add(kept_tokens);
            selected.push(SelectedMessage {
                index: candidate.index,
                class: candidate.class,
                original_tokens: candidate.original_tokens,
                kept_tokens,
                message: trimmed,
            });
        }
    }
}

pub(super) fn pack_selected_messages(
    mut selected: Vec<SelectedMessage>,
    report: &mut ContextBudgetReport,
) -> Vec<ChatMessage> {
    selected.sort_by_key(|entry| entry.index);
    let mut packed = Vec::with_capacity(selected.len());
    for entry in selected {
        report
            .class_mut(entry.class)
            .record_kept(entry.original_tokens, entry.kept_tokens);
        report.post_messages = report.post_messages.saturating_add(1);
        report.post_tokens = report.post_tokens.saturating_add(entry.kept_tokens);
        packed.push(entry.message);
    }
    packed
}
