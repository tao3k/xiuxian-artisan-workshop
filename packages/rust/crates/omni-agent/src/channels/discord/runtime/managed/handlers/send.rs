use std::sync::Arc;

use crate::channels::traits::{Channel, ChannelMessage};

use crate::channels::telegram::runtime::jobs::observability::json_summary::{
    optional_bool_token, optional_u64_token, optional_usize_token, summarize_json_reply,
};

pub(super) async fn send_response(
    channel: &Arc<dyn Channel>,
    recipient: &str,
    response: String,
    msg: &ChannelMessage,
    event: &'static str,
) {
    match channel.send(&response, recipient).await {
        Ok(()) => {
            tracing::info!(
                event,
                session_key = %msg.session_key,
                recipient = %msg.recipient,
                channel_name = channel.name(),
                reply_chars = response.chars().count(),
                reply_bytes = response.len(),
                "discord command reply sent"
            );
            if let Some(json_summary) = summarize_json_reply(&response) {
                tracing::info!(
                    event,
                    session_key = %msg.session_key,
                    recipient = %msg.recipient,
                    channel_name = channel.name(),
                    json_kind = json_summary.kind.as_deref().unwrap_or(""),
                    json_available = optional_bool_token(json_summary.available),
                    json_status = json_summary.status.as_deref().unwrap_or(""),
                    json_found = optional_bool_token(json_summary.found),
                    json_decision = json_summary.decision.as_deref().unwrap_or(""),
                    json_session_scope = json_summary.session_scope.as_deref().unwrap_or(""),
                    json_logical_session_id = json_summary.logical_session_id.as_deref().unwrap_or(""),
                    json_partition_key = json_summary.partition_key.as_deref().unwrap_or(""),
                    json_partition_mode = json_summary.partition_mode.as_deref().unwrap_or(""),
                    json_context_mode = json_summary.context_mode.as_deref().unwrap_or(""),
                    json_saved_snapshot_status =
                        json_summary.saved_snapshot_status.as_deref().unwrap_or(""),
                    json_runtime_backend_ready = optional_bool_token(json_summary.runtime_backend_ready),
                    json_runtime_active_backend =
                        json_summary.runtime_active_backend.as_deref().unwrap_or(""),
                    json_runtime_startup_load_status = json_summary
                        .runtime_startup_load_status
                        .as_deref()
                        .unwrap_or(""),
                    json_admission_enabled = optional_bool_token(json_summary.admission_enabled),
                    json_admission_llm_reject_threshold_pct =
                        optional_u64_token(json_summary.admission_llm_reject_threshold_pct),
                    json_admission_embedding_reject_threshold_pct =
                        optional_u64_token(json_summary.admission_embedding_reject_threshold_pct),
                    json_admission_total = optional_u64_token(json_summary.admission_metrics_total),
                    json_admission_admitted =
                        optional_u64_token(json_summary.admission_metrics_admitted),
                    json_admission_rejected =
                        optional_u64_token(json_summary.admission_metrics_rejected),
                    json_admission_rejected_llm_saturated =
                        optional_u64_token(json_summary.admission_metrics_rejected_llm_saturated),
                    json_admission_rejected_embedding_saturated = optional_u64_token(
                        json_summary.admission_metrics_rejected_embedding_saturated
                    ),
                    json_admission_reject_rate_pct =
                        optional_u64_token(json_summary.admission_metrics_reject_rate_pct),
                    json_result_recalled_injected =
                        optional_u64_token(json_summary.result_recalled_injected),
                    json_query_tokens = optional_u64_token(json_summary.query_tokens),
                    json_override_admin_count = optional_usize_token(json_summary.override_admin_count),
                    json_keys = json_summary.keys,
                    "discord command reply json summary"
                );
            }
        }
        Err(error) => tracing::warn!(
            event,
            error = %error,
            session_key = %msg.session_key,
            recipient = %msg.recipient,
            "discord failed to send command reply"
        ),
    }
}

pub(super) async fn send_completion(
    channel: &Arc<dyn Channel>,
    recipient: &str,
    response: String,
    event: &'static str,
) {
    match channel.send(&response, recipient).await {
        Ok(()) => tracing::info!(
            event,
            recipient = %recipient,
            "discord command completion reply sent"
        ),
        Err(error) => tracing::warn!(
            event,
            error = %error,
            recipient = %recipient,
            "discord failed to send command completion reply"
        ),
    }
}
