use crate::agent::SessionContextBudgetSnapshot;

use super::class_format::{compute_largest_bottlenecks, format_context_budget_class_row};

pub(in super::super::super) fn format_context_budget_snapshot(
    snapshot: &SessionContextBudgetSnapshot,
) -> String {
    let (largest_drop, largest_trunc) = compute_largest_bottlenecks(snapshot);

    let mut lines = vec![
        "============================================================".to_string(),
        "session-budget dashboard".to_string(),
        "============================================================".to_string(),
        "Overview:".to_string(),
        format!("  captured_at_unix_ms={}", snapshot.created_at_unix_ms),
        format!("  strategy={}", snapshot.strategy.as_str()),
        format!(
            "  budget={} reserve={} effective={}",
            snapshot.budget_tokens, snapshot.reserve_tokens, snapshot.effective_budget_tokens
        ),
        format!(
            "  messages={} -> {} (dropped={})",
            snapshot.pre_messages, snapshot.post_messages, snapshot.dropped_messages
        ),
        format!(
            "  tokens={} -> {} (dropped={})",
            snapshot.pre_tokens, snapshot.post_tokens, snapshot.dropped_tokens
        ),
        "------------------------------------------------------------".to_string(),
        "Classes:".to_string(),
        "  class           in_msg  kept  drop  trunc  in_tok  kept   drop   trunc".to_string(),
    ];
    lines.extend(format_context_budget_class_row(
        "non_system",
        &snapshot.non_system,
    ));
    lines.extend(format_context_budget_class_row(
        "regular_system",
        &snapshot.regular_system,
    ));
    lines.extend(format_context_budget_class_row(
        "summary_system",
        &snapshot.summary_system,
    ));
    lines.extend([
        "------------------------------------------------------------".to_string(),
        "Bottlenecks:".to_string(),
        format!(
            "  largest_dropped_tokens={} ({})",
            largest_drop.0, largest_drop.1
        ),
        format!(
            "  largest_truncated_tokens={} ({})",
            largest_trunc.0, largest_trunc.1
        ),
        "============================================================".to_string(),
    ]);
    lines.join("\n")
}
