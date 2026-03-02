use serde_json::json;

use crate::agent::{SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot};

pub(super) fn compute_largest_bottlenecks(
    snapshot: &SessionContextBudgetSnapshot,
) -> ((&'static str, usize), (&'static str, usize)) {
    let classes = [
        ("non_system", snapshot.non_system),
        ("regular_system", snapshot.regular_system),
        ("summary_system", snapshot.summary_system),
    ];

    let mut largest_drop = ("none", 0usize);
    let mut largest_trunc = ("none", 0usize);
    for (name, class) in classes {
        if class.dropped_tokens > largest_drop.1 {
            largest_drop = (name, class.dropped_tokens);
        }
        if class.truncated_tokens > largest_trunc.1 {
            largest_trunc = (name, class.truncated_tokens);
        }
    }

    (largest_drop, largest_trunc)
}

pub(super) fn format_context_budget_class_json(
    stats: &SessionContextBudgetClassSnapshot,
) -> serde_json::Value {
    json!({
        "input_messages": stats.input_messages,
        "kept_messages": stats.kept_messages,
        "dropped_messages": stats.dropped_messages,
        "truncated_messages": stats.truncated_messages,
        "input_tokens": stats.input_tokens,
        "kept_tokens": stats.kept_tokens,
        "dropped_tokens": stats.dropped_tokens,
        "truncated_tokens": stats.truncated_tokens,
    })
}

pub(super) fn format_context_budget_class_row(
    label: &str,
    stats: &SessionContextBudgetClassSnapshot,
) -> Vec<String> {
    vec![format!(
        "  {label:<14} {in_msg:>6} {kept:>5} {drop:>5} {trunc:>6} {in_tok:>7} {kept_tok:>6} {drop_tok:>6} {trunc_tok:>7}",
        in_msg = stats.input_messages,
        kept = stats.kept_messages,
        drop = stats.dropped_messages,
        trunc = stats.truncated_messages,
        in_tok = stats.input_tokens,
        kept_tok = stats.kept_tokens,
        drop_tok = stats.dropped_tokens,
        trunc_tok = stats.truncated_tokens,
    )]
}
