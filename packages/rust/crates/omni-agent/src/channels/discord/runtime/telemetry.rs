use std::time::Duration;

use tokio::sync::mpsc;

use super::foreground::DiscordForegroundSnapshot;
use crate::agent::DownstreamAdmissionRuntimeSnapshot;
use crate::channels::traits::ChannelMessage;

const RUNTIME_SNAPSHOT_INTERVAL_ENV: &str = "OMNI_AGENT_DISCORD_RUNTIME_SNAPSHOT_INTERVAL_SECS";
const DEFAULT_RUNTIME_SNAPSHOT_INTERVAL_SECS: u64 = 30;

pub(super) fn snapshot_interval_from_env() -> Option<Duration> {
    resolve_snapshot_interval_secs(|name| std::env::var(name).ok()).map(Duration::from_secs)
}

pub(super) fn emit_runtime_snapshot(
    mode: &str,
    inbound_tx: &mpsc::Sender<ChannelMessage>,
    inbound_queue_capacity: usize,
    foreground: &DiscordForegroundSnapshot,
    admission: DownstreamAdmissionRuntimeSnapshot,
) {
    let inbound_queue_available = inbound_tx.capacity();
    let inbound_queue_depth = inbound_queue_capacity.saturating_sub(inbound_queue_available);
    tracing::info!(
        event = "discord.runtime.snapshot",
        mode,
        inbound_queue_capacity,
        inbound_queue_depth,
        inbound_queue_available,
        foreground_max_in_flight = foreground.max_in_flight_messages,
        foreground_available_permits = foreground.available_permits,
        foreground_in_flight = foreground.in_flight_messages,
        foreground_task_count = foreground.task_count,
        admission_enabled = admission.enabled,
        admission_llm_reject_threshold_pct = admission.llm_reject_threshold_pct,
        admission_embedding_reject_threshold_pct = admission.embedding_reject_threshold_pct,
        admission_total_evaluations = admission.metrics.total,
        admission_admitted_total = admission.metrics.admitted,
        admission_rejected_total = admission.metrics.rejected,
        admission_rejected_llm_total = admission.metrics.rejected_llm_saturated,
        admission_rejected_embedding_total = admission.metrics.rejected_embedding_saturated,
        admission_reject_rate_pct = admission.metrics.reject_rate_pct,
        "discord runtime snapshot"
    );
}

pub(in crate::channels::discord::runtime) fn resolve_snapshot_interval_secs<F>(
    lookup: F,
) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    let Some(raw) = lookup(RUNTIME_SNAPSHOT_INTERVAL_ENV) else {
        return Some(DEFAULT_RUNTIME_SNAPSHOT_INTERVAL_SECS);
    };
    match raw.trim().parse::<u64>() {
        Ok(0) => None,
        Ok(value) => Some(value),
        Err(_) => {
            tracing::warn!(
                env_var = RUNTIME_SNAPSHOT_INTERVAL_ENV,
                value = %raw,
                default_secs = DEFAULT_RUNTIME_SNAPSHOT_INTERVAL_SECS,
                "invalid discord runtime snapshot interval; using default"
            );
            Some(DEFAULT_RUNTIME_SNAPSHOT_INTERVAL_SECS)
        }
    }
}
