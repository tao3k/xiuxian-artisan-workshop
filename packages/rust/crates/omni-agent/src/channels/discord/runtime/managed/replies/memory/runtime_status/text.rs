use crate::agent::{DownstreamAdmissionRuntimeSnapshot, MemoryRuntimeStatusSnapshot};

use super::super::super::shared::{
    format_optional_f32, format_optional_u32, format_optional_usize,
};
use super::helpers::{
    format_optional_bool, format_optional_str, format_optional_string, format_yes_no,
    is_backend_ready,
};

pub(in super::super) fn format_memory_runtime_status_lines(
    status: MemoryRuntimeStatusSnapshot,
) -> Vec<String> {
    let backend_ready = is_backend_ready(
        status.enabled,
        status.active_backend.is_some(),
        status.startup_load_status,
    );
    vec![
        format!("- `memory_enabled={}`", format_yes_no(status.enabled)),
        format!(
            "- `configured_backend={}`",
            format_optional_string(status.configured_backend)
        ),
        format!(
            "- `active_backend={}`",
            format_optional_str(status.active_backend)
        ),
        format!(
            "- `strict_startup={}`",
            format_optional_bool(status.strict_startup)
        ),
        format!("- `startup_load_status={}`", status.startup_load_status),
        format!("- `backend_ready={}`", format_yes_no(backend_ready)),
        format!(
            "- `store_path={}`",
            format_optional_string(status.store_path)
        ),
        format!(
            "- `table_name={}`",
            format_optional_string(status.table_name)
        ),
        format!(
            "- `gate_promote_threshold={}`",
            format_optional_f32(status.gate_promote_threshold)
        ),
        format!(
            "- `gate_obsolete_threshold={}`",
            format_optional_f32(status.gate_obsolete_threshold)
        ),
        format!(
            "- `gate_promote_min_usage={}`",
            format_optional_u32(status.gate_promote_min_usage)
        ),
        format!(
            "- `gate_obsolete_min_usage={}`",
            format_optional_u32(status.gate_obsolete_min_usage)
        ),
        format!(
            "- `gate_promote_failure_rate_ceiling={}`",
            format_optional_f32(status.gate_promote_failure_rate_ceiling)
        ),
        format!(
            "- `gate_obsolete_failure_rate_floor={}`",
            format_optional_f32(status.gate_obsolete_failure_rate_floor)
        ),
        format!(
            "- `gate_promote_min_ttl_score={}`",
            format_optional_f32(status.gate_promote_min_ttl_score)
        ),
        format!(
            "- `gate_obsolete_max_ttl_score={}`",
            format_optional_f32(status.gate_obsolete_max_ttl_score)
        ),
        format!(
            "- `episodes_total={}`",
            format_optional_usize(status.episodes_total)
        ),
        format!(
            "- `q_values_total={}`",
            format_optional_usize(status.q_values_total)
        ),
    ]
}

pub(in super::super) fn format_downstream_admission_status_lines(
    status: DownstreamAdmissionRuntimeSnapshot,
) -> Vec<String> {
    vec![
        format!("- `enabled={}`", format_yes_no(status.enabled)),
        format!(
            "- `llm_reject_threshold_pct={}` / `embedding_reject_threshold_pct={}`",
            status.llm_reject_threshold_pct, status.embedding_reject_threshold_pct
        ),
        format!(
            "- `total={}` / `admitted={}` / `rejected={}` / `reject_rate_pct={}`",
            status.metrics.total,
            status.metrics.admitted,
            status.metrics.rejected,
            status.metrics.reject_rate_pct
        ),
        format!(
            "- `rejected_llm_saturated={}` / `rejected_embedding_saturated={}`",
            status.metrics.rejected_llm_saturated, status.metrics.rejected_embedding_saturated
        ),
    ]
}
