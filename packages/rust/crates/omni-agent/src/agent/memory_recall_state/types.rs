use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::super::memory_recall::MemoryRecallPlan;

pub(crate) const EMBEDDING_SOURCE_EMBEDDING: &str = "embedding";
pub(crate) const EMBEDDING_SOURCE_EMBEDDING_REPAIRED: &str = "embedding_repaired";
pub(crate) const EMBEDDING_SOURCE_UNAVAILABLE: &str = "embedding_unavailable";
pub(crate) const EMBEDDING_SOURCE_HASH: &str = "hash";
pub(crate) const EMBEDDING_SOURCE_UNKNOWN: &str = "unknown";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMemoryRecallDecision {
    Injected,
    Skipped,
}

impl SessionMemoryRecallDecision {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Injected => "injected",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionMemoryRecallSnapshot {
    pub created_at_unix_ms: u64,
    pub query_tokens: usize,
    pub recall_feedback_bias: f32,
    pub embedding_source: &'static str,
    pub k1: usize,
    pub k2: usize,
    pub lambda: f32,
    pub min_score: f32,
    pub max_context_chars: usize,
    pub budget_pressure: f32,
    pub window_pressure: f32,
    pub effective_budget_tokens: Option<usize>,
    pub active_turns_estimate: usize,
    pub summary_segment_count: usize,
    pub recalled_total: usize,
    pub recalled_selected: usize,
    pub recalled_injected: usize,
    pub context_chars_injected: usize,
    pub best_score: Option<f32>,
    pub weakest_score: Option<f32>,
    pub pipeline_duration_ms: u64,
    pub decision: SessionMemoryRecallDecision,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SessionMemoryRecallSnapshotInput {
    pub(crate) active_turns_estimate: usize,
    pub(crate) summary_segment_count: usize,
    pub(crate) query_tokens: usize,
    pub(crate) recall_feedback_bias: f32,
    pub(crate) embedding_source: &'static str,
    pub(crate) recalled_total: usize,
    pub(crate) recalled_selected: usize,
    pub(crate) recalled_injected: usize,
    pub(crate) context_chars_injected: usize,
    pub(crate) best_score: Option<f32>,
    pub(crate) weakest_score: Option<f32>,
    pub(crate) pipeline_duration_ms: u64,
    pub(crate) decision: SessionMemoryRecallDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct StoredSessionMemoryRecallSnapshot {
    created_at_unix_ms: u64,
    query_tokens: usize,
    #[serde(default)]
    recall_feedback_bias: f32,
    embedding_source: String,
    k1: usize,
    k2: usize,
    lambda: f32,
    min_score: f32,
    max_context_chars: usize,
    budget_pressure: f32,
    window_pressure: f32,
    effective_budget_tokens: Option<usize>,
    active_turns_estimate: usize,
    summary_segment_count: usize,
    recalled_total: usize,
    recalled_selected: usize,
    recalled_injected: usize,
    context_chars_injected: usize,
    best_score: Option<f32>,
    weakest_score: Option<f32>,
    pipeline_duration_ms: u64,
    decision: SessionMemoryRecallDecision,
}

impl SessionMemoryRecallSnapshot {
    pub(crate) fn from_plan(
        plan: MemoryRecallPlan,
        input: SessionMemoryRecallSnapshotInput,
    ) -> Self {
        Self {
            created_at_unix_ms: now_unix_ms(),
            query_tokens: input.query_tokens,
            recall_feedback_bias: input.recall_feedback_bias,
            embedding_source: input.embedding_source,
            k1: plan.k1,
            k2: plan.k2,
            lambda: plan.lambda,
            min_score: plan.min_score,
            max_context_chars: plan.max_context_chars,
            budget_pressure: plan.budget_pressure,
            window_pressure: plan.window_pressure,
            effective_budget_tokens: plan.effective_budget_tokens,
            active_turns_estimate: input.active_turns_estimate,
            summary_segment_count: input.summary_segment_count,
            recalled_total: input.recalled_total,
            recalled_selected: input.recalled_selected,
            recalled_injected: input.recalled_injected,
            context_chars_injected: input.context_chars_injected,
            best_score: input.best_score,
            weakest_score: input.weakest_score,
            pipeline_duration_ms: input.pipeline_duration_ms,
            decision: input.decision,
        }
    }
}

impl From<SessionMemoryRecallSnapshot> for StoredSessionMemoryRecallSnapshot {
    fn from(snapshot: SessionMemoryRecallSnapshot) -> Self {
        Self {
            created_at_unix_ms: snapshot.created_at_unix_ms,
            query_tokens: snapshot.query_tokens,
            recall_feedback_bias: snapshot.recall_feedback_bias,
            embedding_source: snapshot.embedding_source.to_string(),
            k1: snapshot.k1,
            k2: snapshot.k2,
            lambda: snapshot.lambda,
            min_score: snapshot.min_score,
            max_context_chars: snapshot.max_context_chars,
            budget_pressure: snapshot.budget_pressure,
            window_pressure: snapshot.window_pressure,
            effective_budget_tokens: snapshot.effective_budget_tokens,
            active_turns_estimate: snapshot.active_turns_estimate,
            summary_segment_count: snapshot.summary_segment_count,
            recalled_total: snapshot.recalled_total,
            recalled_selected: snapshot.recalled_selected,
            recalled_injected: snapshot.recalled_injected,
            context_chars_injected: snapshot.context_chars_injected,
            best_score: snapshot.best_score,
            weakest_score: snapshot.weakest_score,
            pipeline_duration_ms: snapshot.pipeline_duration_ms,
            decision: snapshot.decision,
        }
    }
}

impl StoredSessionMemoryRecallSnapshot {
    pub(super) fn into_runtime(self) -> SessionMemoryRecallSnapshot {
        SessionMemoryRecallSnapshot {
            created_at_unix_ms: self.created_at_unix_ms,
            query_tokens: self.query_tokens,
            recall_feedback_bias: self.recall_feedback_bias,
            embedding_source: normalize_embedding_source(&self.embedding_source),
            k1: self.k1,
            k2: self.k2,
            lambda: self.lambda,
            min_score: self.min_score,
            max_context_chars: self.max_context_chars,
            budget_pressure: self.budget_pressure,
            window_pressure: self.window_pressure,
            effective_budget_tokens: self.effective_budget_tokens,
            active_turns_estimate: self.active_turns_estimate,
            summary_segment_count: self.summary_segment_count,
            recalled_total: self.recalled_total,
            recalled_selected: self.recalled_selected,
            recalled_injected: self.recalled_injected,
            context_chars_injected: self.context_chars_injected,
            best_score: self.best_score,
            weakest_score: self.weakest_score,
            pipeline_duration_ms: self.pipeline_duration_ms,
            decision: self.decision,
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn normalize_embedding_source(value: &str) -> &'static str {
    match value {
        EMBEDDING_SOURCE_EMBEDDING => EMBEDDING_SOURCE_EMBEDDING,
        EMBEDDING_SOURCE_EMBEDDING_REPAIRED => EMBEDDING_SOURCE_EMBEDDING_REPAIRED,
        EMBEDDING_SOURCE_UNAVAILABLE => EMBEDDING_SOURCE_UNAVAILABLE,
        EMBEDDING_SOURCE_HASH => EMBEDDING_SOURCE_HASH,
        _ => EMBEDDING_SOURCE_UNKNOWN,
    }
}
