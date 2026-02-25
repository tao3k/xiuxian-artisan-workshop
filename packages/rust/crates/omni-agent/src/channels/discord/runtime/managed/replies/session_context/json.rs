use serde_json::json;

use crate::agent::{
    DownstreamAdmissionRuntimeSnapshot, SessionContextSnapshotInfo, SessionContextWindowInfo,
};

use super::mode::format_context_mode;

pub(in super::super::super) fn format_session_context_snapshot_json(
    session_id: &str,
    partition_key: &str,
    partition_mode: &str,
    active: SessionContextWindowInfo,
    snapshot: Option<SessionContextSnapshotInfo>,
    admission: DownstreamAdmissionRuntimeSnapshot,
) -> String {
    let snapshot_json = match snapshot {
        Some(info) => json!({
            "status": "available",
            "saved_messages": info.messages,
            "saved_summary_segments": info.summary_segments,
            "saved_at_unix_ms": info.saved_at_unix_ms,
            "saved_age_secs": info.saved_age_secs,
            "restore_hint": "/resume",
        }),
        None => json!({
            "status": "none",
        }),
    };

    json!({
        "kind": "session_context",
        "logical_session_id": session_id,
        "partition_key": partition_key,
        "partition_mode": partition_mode,
        "mode": format_context_mode(active.mode),
        "active": {
            "messages": active.messages,
            "summary_segments": active.summary_segments,
            "window_turns": active.window_turns,
            "window_slots": active.window_slots,
            "window_tool_calls": active.total_tool_calls,
        },
        "saved_snapshot": snapshot_json,
        "admission": {
            "enabled": admission.enabled,
            "llm_reject_threshold_pct": admission.llm_reject_threshold_pct,
            "embedding_reject_threshold_pct": admission.embedding_reject_threshold_pct,
            "metrics": {
                "total": admission.metrics.total,
                "admitted": admission.metrics.admitted,
                "rejected": admission.metrics.rejected,
                "rejected_llm_saturated": admission.metrics.rejected_llm_saturated,
                "rejected_embedding_saturated": admission.metrics.rejected_embedding_saturated,
                "reject_rate_pct": admission.metrics.reject_rate_pct,
            },
        },
    })
    .to_string()
}
