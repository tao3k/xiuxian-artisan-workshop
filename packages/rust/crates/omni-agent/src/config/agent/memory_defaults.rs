use std::path::PathBuf;

use super::types::MemoryConfig;

pub(super) fn default_memory_path() -> String {
    let root = std::env::var("PRJ_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map_or_else(
            || std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            PathBuf::from,
        );

    let data_home = std::env::var("PRJ_DATA_HOME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map_or_else(|| root.join(".data"), PathBuf::from);

    data_home
        .join("omni-agent")
        .join("memory")
        .to_string_lossy()
        .to_string()
}

pub(super) fn default_embedding_dim() -> usize {
    384
}

pub(super) fn default_memory_table() -> String {
    "episodes".to_string()
}

pub(super) fn default_recall_k1() -> usize {
    20
}

pub(super) fn default_recall_k2() -> usize {
    5
}

pub(super) fn default_recall_lambda() -> f32 {
    0.3
}

pub(super) fn default_memory_persistence_backend() -> String {
    "auto".to_string()
}

pub(super) fn default_memory_persistence_key_prefix() -> String {
    "omni-agent:memory".to_string()
}

pub(super) fn default_recall_credit_enabled() -> bool {
    true
}

pub(super) fn default_recall_credit_max_candidates() -> usize {
    4
}

pub(super) fn default_decay_enabled() -> bool {
    true
}

pub(super) fn default_decay_every_turns() -> usize {
    24
}

pub(super) fn default_decay_factor() -> f32 {
    0.985
}

pub(super) fn default_gate_promote_threshold() -> f32 {
    0.78
}

pub(super) fn default_gate_obsolete_threshold() -> f32 {
    0.32
}

pub(super) fn default_gate_promote_min_usage() -> u32 {
    3
}

pub(super) fn default_gate_obsolete_min_usage() -> u32 {
    2
}

pub(super) fn default_gate_promote_failure_rate_ceiling() -> f32 {
    0.25
}

pub(super) fn default_gate_obsolete_failure_rate_floor() -> f32 {
    0.70
}

pub(super) fn default_gate_promote_min_ttl_score() -> f32 {
    0.50
}

pub(super) fn default_gate_obsolete_max_ttl_score() -> f32 {
    0.45
}

pub(super) fn default_stream_consumer_enabled() -> bool {
    true
}

pub(super) fn default_stream_name() -> String {
    "memory.events".to_string()
}

pub(super) fn default_stream_consumer_group() -> String {
    "omni-agent-memory".to_string()
}

pub(super) fn default_stream_consumer_name_prefix() -> String {
    "agent".to_string()
}

pub(super) fn default_stream_consumer_batch_size() -> usize {
    32
}

pub(super) fn default_stream_consumer_block_ms() -> u64 {
    1000
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            path: default_memory_path(),
            embedding_backend: None,
            embedding_base_url: None,
            embedding_model: None,
            embedding_batch_max_size: None,
            embedding_batch_max_concurrency: None,
            embedding_timeout_ms: None,
            embedding_timeout_cooldown_ms: None,
            embedding_dim: default_embedding_dim(),
            table_name: default_memory_table(),
            recall_k1: default_recall_k1(),
            recall_k2: default_recall_k2(),
            recall_lambda: default_recall_lambda(),
            persistence_backend: default_memory_persistence_backend(),
            persistence_valkey_url: None,
            persistence_key_prefix: default_memory_persistence_key_prefix(),
            persistence_strict_startup: None,
            recall_credit_enabled: default_recall_credit_enabled(),
            recall_credit_max_candidates: default_recall_credit_max_candidates(),
            decay_enabled: default_decay_enabled(),
            decay_every_turns: default_decay_every_turns(),
            decay_factor: default_decay_factor(),
            gate_promote_threshold: default_gate_promote_threshold(),
            gate_obsolete_threshold: default_gate_obsolete_threshold(),
            gate_promote_min_usage: default_gate_promote_min_usage(),
            gate_obsolete_min_usage: default_gate_obsolete_min_usage(),
            gate_promote_failure_rate_ceiling: default_gate_promote_failure_rate_ceiling(),
            gate_obsolete_failure_rate_floor: default_gate_obsolete_failure_rate_floor(),
            gate_promote_min_ttl_score: default_gate_promote_min_ttl_score(),
            gate_obsolete_max_ttl_score: default_gate_obsolete_max_ttl_score(),
            stream_consumer_enabled: default_stream_consumer_enabled(),
            stream_name: default_stream_name(),
            stream_consumer_group: default_stream_consumer_group(),
            stream_consumer_name_prefix: default_stream_consumer_name_prefix(),
            stream_consumer_batch_size: default_stream_consumer_batch_size(),
            stream_consumer_block_ms: default_stream_consumer_block_ms(),
        }
    }
}
