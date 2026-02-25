mod consolidation;
mod turn_store;

use omni_memory::{EpisodeStore, MemoryGatePolicy};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::observability::SessionEvent;

use super::Agent;
use super::memory::{sanitize_decay_factor, should_apply_decay};
use super::memory_state::MemoryStateBackend;

fn encode_string_list_for_stream(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string())
}

fn stream_excerpt(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let mut out = String::new();
    for ch in value.chars().take(max_chars) {
        out.push(ch);
    }
    out
}

fn persist_memory_state(
    backend: Option<&Arc<MemoryStateBackend>>,
    store: &EpisodeStore,
    session_id: &str,
    reason: &str,
) {
    let Some(backend) = backend else {
        return;
    };
    let started = Instant::now();
    match backend.save(store) {
        Ok(()) => {
            tracing::debug!(
                event = SessionEvent::MemoryStateSaveSucceeded.as_str(),
                backend = backend.backend_name(),
                session_id,
                reason,
                episodes = store.len(),
                q_values = store.q_table.len(),
                duration_ms = started.elapsed().as_millis(),
                "memory state persisted"
            );
        }
        Err(error) => {
            tracing::warn!(
                event = SessionEvent::MemoryStateSaveFailed.as_str(),
                backend = backend.backend_name(),
                session_id,
                reason,
                duration_ms = started.elapsed().as_millis(),
                error = %error,
                "failed to persist memory state"
            );
        }
    }
}

impl Agent {
    fn memory_gate_policy(&self) -> MemoryGatePolicy {
        let mut policy = MemoryGatePolicy::default();
        let Some(memory_cfg) = self.config.memory.as_ref() else {
            return policy;
        };

        policy.promote_threshold = memory_cfg.gate_promote_threshold.clamp(0.0, 1.0);
        policy.obsolete_threshold = memory_cfg.gate_obsolete_threshold.clamp(0.0, 1.0);
        policy.promote_min_usage = memory_cfg.gate_promote_min_usage.max(1);
        policy.obsolete_min_usage = memory_cfg.gate_obsolete_min_usage.max(1);
        policy.promote_failure_rate_ceiling =
            memory_cfg.gate_promote_failure_rate_ceiling.clamp(0.0, 1.0);
        policy.obsolete_failure_rate_floor =
            memory_cfg.gate_obsolete_failure_rate_floor.clamp(0.0, 1.0);
        policy.promote_min_ttl_score = memory_cfg.gate_promote_min_ttl_score.clamp(0.0, 1.0);
        policy.obsolete_max_ttl_score = memory_cfg.gate_obsolete_max_ttl_score.clamp(0.0, 1.0);
        policy
    }

    pub(super) fn memory_stream_name(&self) -> &str {
        self.config
            .memory
            .as_ref()
            .map(|cfg| cfg.stream_name.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("memory.events")
    }

    async fn publish_memory_stream_event(&self, fields: Vec<(String, String)>) {
        if let Err(error) = self
            .session
            .publish_stream_event(self.memory_stream_name(), fields)
            .await
        {
            tracing::warn!(
                error = %error,
                "failed to publish memory stream event"
            );
        }
    }

    fn maybe_apply_memory_decay(&self, session_id: &str, store: &EpisodeStore) {
        let Some(memory_cfg) = self.config.memory.as_ref() else {
            return;
        };
        let turn_index = self
            .memory_decay_turn_counter
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        if !should_apply_decay(
            memory_cfg.decay_enabled,
            memory_cfg.decay_every_turns,
            turn_index,
        ) {
            return;
        }
        let decay_factor = sanitize_decay_factor(memory_cfg.decay_factor);
        let started = Instant::now();
        store.apply_decay(decay_factor);
        persist_memory_state(
            self.memory_state_backend.as_ref(),
            store,
            session_id,
            "decay",
        );
        tracing::debug!(
            event = SessionEvent::MemoryDecayApplied.as_str(),
            session_id,
            turn_index,
            decay_every_turns = memory_cfg.decay_every_turns,
            decay_factor,
            duration_ms = started.elapsed().as_millis(),
            "memory decay applied"
        );
    }
}
