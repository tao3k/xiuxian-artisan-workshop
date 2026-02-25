use omni_agent::{MemoryConfig, RuntimeSettings};

use super::super::shared::normalize_unit_f32;
use super::embedding::apply_memory_runtime_embedding_settings;

pub(super) fn apply_memory_runtime_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    apply_memory_runtime_embedding_settings(memory, runtime_settings);
    apply_memory_runtime_persistence_settings(memory, runtime_settings);
    apply_memory_runtime_recall_gate_settings(memory, runtime_settings);
    apply_memory_runtime_stream_settings(memory, runtime_settings);
}

fn apply_memory_runtime_persistence_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(backend) = runtime_settings
        .memory
        .persistence_backend
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.persistence_backend = backend.to_string();
    }
    if let Some(url) = runtime_settings
        .session
        .valkey_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.persistence_valkey_url = Some(url.to_string());
    }
    if let Some(prefix) = runtime_settings
        .memory
        .persistence_key_prefix
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.persistence_key_prefix = prefix.to_string();
    }
    if let Some(strict_startup) = runtime_settings.memory.persistence_strict_startup {
        memory.persistence_strict_startup = Some(strict_startup);
    }
}

fn apply_memory_runtime_recall_gate_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(enabled) = runtime_settings.memory.recall_credit_enabled {
        memory.recall_credit_enabled = enabled;
    }
    if let Some(max_candidates) = runtime_settings
        .memory
        .recall_credit_max_candidates
        .filter(|value| *value > 0)
    {
        memory.recall_credit_max_candidates = max_candidates;
    }
    if let Some(enabled) = runtime_settings.memory.decay_enabled {
        memory.decay_enabled = enabled;
    }
    if let Some(every_turns) = runtime_settings
        .memory
        .decay_every_turns
        .filter(|value| *value > 0)
    {
        memory.decay_every_turns = every_turns;
    }
    if let Some(factor) = runtime_settings
        .memory
        .decay_factor
        .filter(|value| *value > 0.0)
    {
        memory.decay_factor = factor;
    }

    if let Some(threshold) = runtime_settings
        .memory
        .gate_promote_threshold
        .and_then(|value| normalize_unit_f32(value, "memory.gate_promote_threshold"))
    {
        memory.gate_promote_threshold = threshold;
    }
    if let Some(threshold) = runtime_settings
        .memory
        .gate_obsolete_threshold
        .and_then(|value| normalize_unit_f32(value, "memory.gate_obsolete_threshold"))
    {
        memory.gate_obsolete_threshold = threshold;
    }
    if let Some(min_usage) = runtime_settings
        .memory
        .gate_promote_min_usage
        .filter(|value| *value > 0)
    {
        memory.gate_promote_min_usage = min_usage;
    }
    if let Some(min_usage) = runtime_settings
        .memory
        .gate_obsolete_min_usage
        .filter(|value| *value > 0)
    {
        memory.gate_obsolete_min_usage = min_usage;
    }
    if let Some(rate) = runtime_settings
        .memory
        .gate_promote_failure_rate_ceiling
        .and_then(|value| normalize_unit_f32(value, "memory.gate_promote_failure_rate_ceiling"))
    {
        memory.gate_promote_failure_rate_ceiling = rate;
    }
    if let Some(rate) = runtime_settings
        .memory
        .gate_obsolete_failure_rate_floor
        .and_then(|value| normalize_unit_f32(value, "memory.gate_obsolete_failure_rate_floor"))
    {
        memory.gate_obsolete_failure_rate_floor = rate;
    }
    if let Some(score) = runtime_settings
        .memory
        .gate_promote_min_ttl_score
        .and_then(|value| normalize_unit_f32(value, "memory.gate_promote_min_ttl_score"))
    {
        memory.gate_promote_min_ttl_score = score;
    }
    if let Some(score) = runtime_settings
        .memory
        .gate_obsolete_max_ttl_score
        .and_then(|value| normalize_unit_f32(value, "memory.gate_obsolete_max_ttl_score"))
    {
        memory.gate_obsolete_max_ttl_score = score;
    }
}

fn apply_memory_runtime_stream_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(enabled) = runtime_settings.memory.stream_consumer_enabled {
        memory.stream_consumer_enabled = enabled;
    }
    if let Some(stream_name) = runtime_settings
        .memory
        .stream_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.stream_name = stream_name.to_string();
    }
    if let Some(consumer_group) = runtime_settings
        .memory
        .stream_consumer_group
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.stream_consumer_group = consumer_group.to_string();
    }
    if let Some(consumer_name_prefix) = runtime_settings
        .memory
        .stream_consumer_name_prefix
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.stream_consumer_name_prefix = consumer_name_prefix.to_string();
    }
    if let Some(batch_size) = runtime_settings
        .memory
        .stream_consumer_batch_size
        .filter(|value| *value > 0)
    {
        memory.stream_consumer_batch_size = batch_size;
    }
    if let Some(block_ms) = runtime_settings
        .memory
        .stream_consumer_block_ms
        .filter(|value| *value > 0)
    {
        memory.stream_consumer_block_ms = block_ms;
    }
}
