use std::time::{SystemTime, UNIX_EPOCH};

use num_traits::ToPrimitive;

use super::Agent;
use super::memory_recall_state::SessionMemoryRecallDecision;

/// Histogram bucket counters for memory-recall pipeline latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MemoryRecallLatencyBucketsSnapshot {
    /// Count of pipelines completed in <= 10ms.
    pub le_10ms: u64,
    /// Count of pipelines completed in <= 25ms.
    pub le_25ms: u64,
    /// Count of pipelines completed in <= 50ms.
    pub le_50ms: u64,
    /// Count of pipelines completed in <= 100ms.
    pub le_100ms: u64,
    /// Count of pipelines completed in <= 250ms.
    pub le_250ms: u64,
    /// Count of pipelines completed in <= 500ms.
    pub le_500ms: u64,
    /// Count of pipelines completed in > 500ms.
    pub gt_500ms: u64,
}

/// Memory-recall metrics snapshot exposed for observability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemoryRecallMetricsSnapshot {
    /// Capture timestamp in Unix milliseconds.
    pub captured_at_unix_ms: u64,
    /// Total number of recall plans attempted.
    pub planned_total: u64,
    /// Total number of injected recall outcomes.
    pub injected_total: u64,
    /// Total number of skipped recall outcomes.
    pub skipped_total: u64,
    /// Total number of completed plans (`injected + skipped`).
    pub completed_total: u64,
    /// Total number of selected candidate episodes.
    pub selected_total: u64,
    /// Total number of injected candidate episodes.
    pub injected_items_total: u64,
    /// Total number of injected context characters.
    pub context_chars_injected_total: u64,
    /// Total pipeline duration across completed plans in milliseconds.
    pub pipeline_duration_ms_total: u64,
    /// Average pipeline duration in milliseconds.
    pub avg_pipeline_duration_ms: f32,
    /// Average selected candidates per completed plan.
    pub avg_selected_per_completed: f32,
    /// Average injected candidates per injected plan.
    pub avg_injected_per_injected: f32,
    /// Injected ratio over completed plans.
    pub injected_rate: f32,
    /// Latency histogram buckets.
    pub latency_buckets: MemoryRecallLatencyBucketsSnapshot,
    /// Total successful embedding calls used by recall.
    pub embedding_success_total: u64,
    /// Total embedding timeout events.
    pub embedding_timeout_total: u64,
    /// Total embedding cooldown reject events.
    pub embedding_cooldown_reject_total: u64,
    /// Total embedding unavailable events.
    pub embedding_unavailable_total: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MemoryRecallMetricsState {
    planned_total: u64,
    injected_total: u64,
    skipped_total: u64,
    selected_total: u64,
    injected_items_total: u64,
    context_chars_injected_total: u64,
    pipeline_duration_ms_total: u64,
    latency_buckets: MemoryRecallLatencyBucketsSnapshot,
    embedding_success_total: u64,
    embedding_timeout_total: u64,
    embedding_cooldown_reject_total: u64,
    embedding_unavailable_total: u64,
}

impl MemoryRecallMetricsState {
    fn observe_plan(&mut self) {
        self.planned_total = self.planned_total.saturating_add(1);
    }

    fn observe_result(
        &mut self,
        decision: SessionMemoryRecallDecision,
        recalled_selected: usize,
        recalled_injected: usize,
        context_chars_injected: usize,
        pipeline_duration_ms: u64,
    ) {
        match decision {
            SessionMemoryRecallDecision::Injected => {
                self.injected_total = self.injected_total.saturating_add(1);
            }
            SessionMemoryRecallDecision::Skipped => {
                self.skipped_total = self.skipped_total.saturating_add(1);
            }
        }

        self.selected_total = self.selected_total.saturating_add(recalled_selected as u64);
        self.injected_items_total = self
            .injected_items_total
            .saturating_add(recalled_injected as u64);
        self.context_chars_injected_total = self
            .context_chars_injected_total
            .saturating_add(context_chars_injected as u64);
        self.pipeline_duration_ms_total = self
            .pipeline_duration_ms_total
            .saturating_add(pipeline_duration_ms);
        self.observe_latency_bucket(pipeline_duration_ms);
    }

    fn observe_latency_bucket(&mut self, duration_ms: u64) {
        if duration_ms <= 10 {
            self.latency_buckets.le_10ms = self.latency_buckets.le_10ms.saturating_add(1);
        } else if duration_ms <= 25 {
            self.latency_buckets.le_25ms = self.latency_buckets.le_25ms.saturating_add(1);
        } else if duration_ms <= 50 {
            self.latency_buckets.le_50ms = self.latency_buckets.le_50ms.saturating_add(1);
        } else if duration_ms <= 100 {
            self.latency_buckets.le_100ms = self.latency_buckets.le_100ms.saturating_add(1);
        } else if duration_ms <= 250 {
            self.latency_buckets.le_250ms = self.latency_buckets.le_250ms.saturating_add(1);
        } else if duration_ms <= 500 {
            self.latency_buckets.le_500ms = self.latency_buckets.le_500ms.saturating_add(1);
        } else {
            self.latency_buckets.gt_500ms = self.latency_buckets.gt_500ms.saturating_add(1);
        }
    }

    fn observe_embedding_success(&mut self) {
        self.embedding_success_total = self.embedding_success_total.saturating_add(1);
    }

    fn observe_embedding_timeout(&mut self) {
        self.embedding_timeout_total = self.embedding_timeout_total.saturating_add(1);
    }

    fn observe_embedding_cooldown_reject(&mut self) {
        self.embedding_cooldown_reject_total =
            self.embedding_cooldown_reject_total.saturating_add(1);
    }

    fn observe_embedding_unavailable(&mut self) {
        self.embedding_unavailable_total = self.embedding_unavailable_total.saturating_add(1);
    }

    fn snapshot(self) -> MemoryRecallMetricsSnapshot {
        let completed_total = self.injected_total.saturating_add(self.skipped_total);
        MemoryRecallMetricsSnapshot {
            captured_at_unix_ms: now_unix_ms(),
            planned_total: self.planned_total,
            injected_total: self.injected_total,
            skipped_total: self.skipped_total,
            completed_total,
            selected_total: self.selected_total,
            injected_items_total: self.injected_items_total,
            context_chars_injected_total: self.context_chars_injected_total,
            pipeline_duration_ms_total: self.pipeline_duration_ms_total,
            avg_pipeline_duration_ms: ratio_as_f32(
                self.pipeline_duration_ms_total,
                completed_total,
            ),
            avg_selected_per_completed: ratio_as_f32(self.selected_total, completed_total),
            avg_injected_per_injected: ratio_as_f32(self.injected_items_total, self.injected_total),
            injected_rate: ratio_as_f32(self.injected_total, completed_total),
            latency_buckets: self.latency_buckets,
            embedding_success_total: self.embedding_success_total,
            embedding_timeout_total: self.embedding_timeout_total,
            embedding_cooldown_reject_total: self.embedding_cooldown_reject_total,
            embedding_unavailable_total: self.embedding_unavailable_total,
        }
    }
}

fn ratio_as_f32(numerator: u64, denominator: u64) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        let numerator = numerator.to_f32().unwrap_or(f32::MAX);
        let denominator = denominator.to_f32().unwrap_or(f32::MAX);
        numerator / denominator
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}

impl Agent {
    pub(crate) async fn record_memory_recall_plan_metrics(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_plan();
    }

    pub(crate) async fn record_memory_recall_result_metrics(
        &self,
        decision: SessionMemoryRecallDecision,
        recalled_selected: usize,
        recalled_injected: usize,
        context_chars_injected: usize,
        pipeline_duration_ms: u64,
    ) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_result(
            decision,
            recalled_selected,
            recalled_injected,
            context_chars_injected,
            pipeline_duration_ms,
        );
    }

    pub(crate) async fn record_memory_embedding_success_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_success();
    }

    pub(crate) async fn record_memory_embedding_timeout_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_timeout();
    }

    pub(crate) async fn record_memory_embedding_cooldown_reject_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_cooldown_reject();
    }

    pub(crate) async fn record_memory_embedding_unavailable_metric(&self) {
        let mut guard = self.memory_recall_metrics.write().await;
        guard.observe_embedding_unavailable();
    }

    /// Return current memory-recall metrics snapshot.
    pub async fn inspect_memory_recall_metrics(&self) -> MemoryRecallMetricsSnapshot {
        let guard = self.memory_recall_metrics.read().await;
        (*guard).snapshot()
    }
}
