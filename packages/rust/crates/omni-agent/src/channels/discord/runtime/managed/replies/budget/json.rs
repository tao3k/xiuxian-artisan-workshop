use serde_json::json;

use crate::agent::SessionContextBudgetSnapshot;

use super::class_format::{compute_largest_bottlenecks, format_context_budget_class_json};

pub(in super::super::super) fn format_context_budget_snapshot_json(
    snapshot: &SessionContextBudgetSnapshot,
) -> String {
    let (largest_drop, largest_trunc) = compute_largest_bottlenecks(snapshot);

    json!({
        "kind": "session_budget",
        "available": true,
        "captured_at_unix_ms": snapshot.created_at_unix_ms,
        "strategy": snapshot.strategy.as_str(),
        "budget_tokens": snapshot.budget_tokens,
        "reserve_tokens": snapshot.reserve_tokens,
        "effective_budget_tokens": snapshot.effective_budget_tokens,
        "messages": {
            "pre": snapshot.pre_messages,
            "post": snapshot.post_messages,
            "dropped": snapshot.dropped_messages,
        },
        "tokens": {
            "pre": snapshot.pre_tokens,
            "post": snapshot.post_tokens,
            "dropped": snapshot.dropped_tokens,
        },
        "classes": {
            "non_system": format_context_budget_class_json(&snapshot.non_system),
            "regular_system": format_context_budget_class_json(&snapshot.regular_system),
            "summary_system": format_context_budget_class_json(&snapshot.summary_system),
        },
        "bottlenecks": {
            "largest_dropped_tokens": {"class": largest_drop.0, "tokens": largest_drop.1},
            "largest_truncated_tokens": {"class": largest_trunc.0, "tokens": largest_trunc.1},
        },
    })
    .to_string()
}

pub(in super::super::super) fn format_context_budget_not_found_json() -> String {
    json!({
        "kind": "session_budget",
        "available": false,
        "status": "not_found",
        "hint": "Run at least one normal turn first (non-command message).",
    })
    .to_string()
}
