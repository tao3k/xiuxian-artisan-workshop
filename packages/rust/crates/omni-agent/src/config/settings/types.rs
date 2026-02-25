use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RuntimeSettings {
    #[serde(default)]
    pub agent: AgentSettings,
    #[serde(default)]
    pub inference: InferenceSettings,
    #[serde(default)]
    pub mcp: McpSettings,
    #[serde(default)]
    pub telegram: TelegramSettings,
    #[serde(default)]
    pub discord: DiscordSettings,
    #[serde(default)]
    pub session: SessionSettings,
    #[serde(default)]
    pub embedding: EmbeddingSettings,
    #[serde(default)]
    pub memory: MemorySettings,
    #[serde(default)]
    pub mistral: MistralSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentSettings {
    pub llm_backend: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct InferenceSettings {
    pub provider: Option<String>,
    pub api_key_env: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub timeout: Option<u64>,
    pub max_tokens: Option<u64>,
    pub max_in_flight: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramSettings {
    #[serde(default)]
    pub acl: TelegramAclSettings,
    pub session_admin_persist: Option<bool>,
    pub group_policy: Option<String>,
    pub group_allow_from: Option<String>,
    pub require_mention: Option<bool>,
    pub groups: Option<HashMap<String, TelegramGroupSettings>>,
    pub mode: Option<String>,
    pub webhook_bind: Option<String>,
    pub webhook_path: Option<String>,
    pub webhook_dedup_backend: Option<String>,
    pub webhook_dedup_ttl_secs: Option<u64>,
    pub webhook_dedup_key_prefix: Option<String>,
    pub max_tool_rounds: Option<u32>,
    pub session_partition: Option<String>,
    pub inbound_queue_capacity: Option<usize>,
    pub foreground_queue_capacity: Option<usize>,
    pub foreground_max_in_flight_messages: Option<usize>,
    pub foreground_turn_timeout_secs: Option<u64>,
    pub foreground_session_gate_backend: Option<String>,
    pub foreground_session_gate_key_prefix: Option<String>,
    pub foreground_session_gate_lease_ttl_secs: Option<u64>,
    pub foreground_session_gate_acquire_timeout_secs: Option<u64>,
    pub send_rate_limit_gate_key_prefix: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclSettings {
    pub allow: Option<TelegramAclAllowSettings>,
    pub admin: Option<TelegramAclPrincipalSettings>,
    pub control: Option<TelegramAclControlSettings>,
    pub slash: Option<TelegramAclSlashSettings>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclAllowSettings {
    pub users: Option<Vec<String>>,
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclPrincipalSettings {
    pub users: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclControlSettings {
    pub allow_from: Option<TelegramAclPrincipalSettings>,
    pub rules: Option<Vec<TelegramAclRuleSettings>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclRuleSettings {
    pub commands: Vec<String>,
    #[serde(default)]
    pub allow: TelegramAclPrincipalSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramAclSlashSettings {
    pub global: Option<TelegramAclPrincipalSettings>,
    pub session_status: Option<TelegramAclPrincipalSettings>,
    pub session_budget: Option<TelegramAclPrincipalSettings>,
    pub session_memory: Option<TelegramAclPrincipalSettings>,
    pub session_feedback: Option<TelegramAclPrincipalSettings>,
    pub job_status: Option<TelegramAclPrincipalSettings>,
    pub jobs_summary: Option<TelegramAclPrincipalSettings>,
    pub background_submit: Option<TelegramAclPrincipalSettings>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramGroupSettings {
    pub enabled: Option<bool>,
    pub group_policy: Option<String>,
    pub allow_from: Option<TelegramAclPrincipalSettings>,
    pub admin_users: Option<TelegramAclPrincipalSettings>,
    pub require_mention: Option<bool>,
    pub topics: Option<HashMap<String, TelegramTopicSettings>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramTopicSettings {
    pub enabled: Option<bool>,
    pub group_policy: Option<String>,
    pub allow_from: Option<TelegramAclPrincipalSettings>,
    pub admin_users: Option<TelegramAclPrincipalSettings>,
    pub require_mention: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordSettings {
    #[serde(default)]
    pub acl: DiscordAclSettings,
    pub runtime_mode: Option<String>,
    pub ingress_bind: Option<String>,
    pub ingress_path: Option<String>,
    pub ingress_secret_token: Option<String>,
    pub session_partition: Option<String>,
    pub inbound_queue_capacity: Option<usize>,
    pub turn_timeout_secs: Option<u64>,
    pub foreground_max_in_flight_messages: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclSettings {
    pub role_aliases: Option<HashMap<String, String>>,
    pub allow: Option<DiscordAclAllowSettings>,
    pub admin: Option<DiscordAclPrincipalSettings>,
    pub control: Option<DiscordAclControlSettings>,
    pub slash: Option<DiscordAclSlashSettings>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclAllowSettings {
    pub users: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    pub guilds: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclPrincipalSettings {
    pub users: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclControlSettings {
    pub allow_from: Option<DiscordAclPrincipalSettings>,
    pub rules: Option<Vec<DiscordAclRuleSettings>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclRuleSettings {
    pub commands: Vec<String>,
    #[serde(default)]
    pub allow: DiscordAclPrincipalSettings,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DiscordAclSlashSettings {
    pub global: Option<DiscordAclPrincipalSettings>,
    pub session_status: Option<DiscordAclPrincipalSettings>,
    pub session_budget: Option<DiscordAclPrincipalSettings>,
    pub session_memory: Option<DiscordAclPrincipalSettings>,
    pub session_feedback: Option<DiscordAclPrincipalSettings>,
    pub job_status: Option<DiscordAclPrincipalSettings>,
    pub jobs_summary: Option<DiscordAclPrincipalSettings>,
    pub background_submit: Option<DiscordAclPrincipalSettings>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct McpSettings {
    pub agent_pool_size: Option<usize>,
    pub agent_handshake_timeout_secs: Option<u64>,
    pub agent_connect_retries: Option<u32>,
    pub agent_strict_startup: Option<bool>,
    pub agent_connect_retry_backoff_ms: Option<u64>,
    pub agent_tool_timeout_secs: Option<u64>,
    pub agent_list_tools_cache_ttl_ms: Option<u64>,
    pub agent_discover_cache_enabled: Option<bool>,
    pub agent_discover_cache_key_prefix: Option<String>,
    pub agent_discover_cache_ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SessionSettings {
    pub window_max_turns: Option<usize>,
    pub consolidation_threshold_turns: Option<usize>,
    pub consolidation_take_turns: Option<usize>,
    pub consolidation_async: Option<bool>,
    pub context_budget_tokens: Option<usize>,
    pub context_budget_reserve_tokens: Option<usize>,
    pub context_budget_strategy: Option<String>,
    pub summary_max_segments: Option<usize>,
    pub summary_max_chars: Option<usize>,
    pub valkey_url: Option<String>,
    pub redis_prefix: Option<String>,
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MemorySettings {
    pub path: Option<String>,
    pub embedding_backend: Option<String>,
    pub embedding_base_url: Option<String>,
    pub embedding_model: Option<String>,
    pub embedding_timeout_ms: Option<u64>,
    pub embedding_timeout_cooldown_ms: Option<u64>,
    pub embedding_dim: Option<usize>,
    pub persistence_backend: Option<String>,
    pub persistence_valkey_url: Option<String>,
    pub persistence_key_prefix: Option<String>,
    pub persistence_strict_startup: Option<bool>,
    pub recall_credit_enabled: Option<bool>,
    pub recall_credit_max_candidates: Option<usize>,
    pub decay_enabled: Option<bool>,
    pub decay_every_turns: Option<usize>,
    pub decay_factor: Option<f32>,
    pub gate_promote_threshold: Option<f32>,
    pub gate_obsolete_threshold: Option<f32>,
    pub gate_promote_min_usage: Option<u32>,
    pub gate_obsolete_min_usage: Option<u32>,
    pub gate_promote_failure_rate_ceiling: Option<f32>,
    pub gate_obsolete_failure_rate_floor: Option<f32>,
    pub gate_promote_min_ttl_score: Option<f32>,
    pub gate_obsolete_max_ttl_score: Option<f32>,
    pub stream_consumer_enabled: Option<bool>,
    pub stream_name: Option<String>,
    pub stream_consumer_group: Option<String>,
    pub stream_consumer_name_prefix: Option<String>,
    pub stream_consumer_batch_size: Option<usize>,
    pub stream_consumer_block_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EmbeddingSettings {
    pub backend: Option<String>,
    #[serde(alias = "timeout")]
    pub timeout_secs: Option<u64>,
    pub max_in_flight: Option<usize>,
    pub batch_max_size: Option<usize>,
    pub batch_max_concurrency: Option<usize>,
    pub model: Option<String>,
    pub litellm_model: Option<String>,
    pub litellm_api_base: Option<String>,
    pub dimension: Option<usize>,
    pub client_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MistralSettings {
    pub enabled: Option<bool>,
    pub auto_start: Option<bool>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub base_url: Option<String>,
    pub startup_timeout_secs: Option<u64>,
    pub probe_timeout_ms: Option<u64>,
    pub probe_interval_ms: Option<u64>,
}
