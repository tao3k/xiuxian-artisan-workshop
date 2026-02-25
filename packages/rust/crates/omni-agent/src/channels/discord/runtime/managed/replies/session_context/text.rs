use crate::agent::{
    DownstreamAdmissionRuntimeSnapshot, SessionContextSnapshotInfo, SessionContextWindowInfo,
};

use super::mode::format_context_mode;

pub(in super::super::super) fn format_session_context_snapshot(
    session_id: &str,
    partition_key: &str,
    partition_mode: &str,
    active: SessionContextWindowInfo,
    snapshot: Option<SessionContextSnapshotInfo>,
    admission: DownstreamAdmissionRuntimeSnapshot,
) -> String {
    let mut lines = vec![
        "============================================================".to_string(),
        "session-context dashboard".to_string(),
        "============================================================".to_string(),
        "Overview:".to_string(),
        format!("  logical_session_id={session_id}"),
        format!("  partition_key={partition_key}"),
        format!("  partition_mode={partition_mode}"),
        format!("  mode={}", format_context_mode(active.mode)),
        "------------------------------------------------------------".to_string(),
        "Active:".to_string(),
        format!("  messages={}", active.messages),
        format!("  summary_segments={}", active.summary_segments),
    ];
    if let Some(window_turns) = active.window_turns {
        lines.push(format!("  window_turns={window_turns}"));
    }
    if let Some(window_slots) = active.window_slots {
        lines.push(format!("  window_slots={window_slots}"));
    }
    if let Some(total_tool_calls) = active.total_tool_calls {
        lines.push(format!("  window_tool_calls={total_tool_calls}"));
    }
    lines.push("------------------------------------------------------------".to_string());
    lines.push("Saved Snapshot:".to_string());
    match snapshot {
        Some(info) => {
            lines.push("  status=available".to_string());
            lines.push(format!("  saved_messages={}", info.messages));
            lines.push(format!(
                "  saved_summary_segments={}",
                info.summary_segments
            ));
            if let Some(saved_at_unix_ms) = info.saved_at_unix_ms {
                lines.push(format!("  saved_at_unix_ms={saved_at_unix_ms}"));
            }
            if let Some(saved_age_secs) = info.saved_age_secs {
                lines.push(format!("  saved_age_secs={saved_age_secs}"));
            }
            lines.push("  restore_hint=/resume".to_string());
        }
        None => {
            lines.push("  status=none".to_string());
        }
    }
    lines.push("------------------------------------------------------------".to_string());
    lines.push("Admission:".to_string());
    lines.push(format!("  enabled={}", admission.enabled));
    lines.push(format!(
        "  llm_reject_threshold_pct={}",
        admission.llm_reject_threshold_pct
    ));
    lines.push(format!(
        "  embedding_reject_threshold_pct={}",
        admission.embedding_reject_threshold_pct
    ));
    lines.push(format!("  total={}", admission.metrics.total));
    lines.push(format!("  admitted={}", admission.metrics.admitted));
    lines.push(format!("  rejected={}", admission.metrics.rejected));
    lines.push(format!(
        "  reject_rate_pct={}",
        admission.metrics.reject_rate_pct
    ));
    lines.push(format!(
        "  rejected_llm_saturated={}",
        admission.metrics.rejected_llm_saturated
    ));
    lines.push(format!(
        "  rejected_embedding_saturated={}",
        admission.metrics.rejected_embedding_saturated
    ));
    lines.push("============================================================".to_string());
    lines.join("\n")
}
