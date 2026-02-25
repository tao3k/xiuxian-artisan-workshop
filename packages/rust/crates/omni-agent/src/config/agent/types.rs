use serde::{Deserialize, Serialize};

use super::{agent_defaults, memory_defaults};

/// One MCP server entry (e.g. SSE URL or stdio command).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    /// Display name for logging.
    pub name: String,
    /// For Streamable HTTP: full URL (e.g. `http://127.0.0.1:3002/sse`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// For stdio: command to spawn (e.g. `omni` with args `["mcp", "--transport", "stdio"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// For stdio: arguments to the command (e.g. `["mcp", "--transport", "stdio"]`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Optional memory (omni-memory) config for two-phase recall and episode storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to the memory store (directory).
    #[serde(default = "memory_defaults::default_memory_path")]
    pub path: String,
    /// Optional embedding backend override for memory runtime.
    ///
    /// Supported values:
    /// - `http`: legacy `/embed/batch` endpoint
    /// - `openai_http`: generic OpenAI-compatible `/v1/embeddings` endpoint
    /// - `mistral_local` (aliases: `mistral_rs`, `mistral_server`): local `mistralrs-server` runtime endpoint
    ///   (typically local model serving; API key is upstream-policy dependent)
    /// - `litellm_rs`: Rust `litellm-rs` provider path
    ///   (provider/API-key oriented; no-key mode stays on Rust HTTP paths)
    ///
    /// Default:
    /// - `litellm_rs` when feature `agent-provider-litellm` is enabled.
    /// - `http` when that feature is disabled.
    ///
    /// When unset, backend selection follows runtime settings / environment defaults.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_backend: Option<String>,
    /// Optional embedding client base URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_base_url: Option<String>,
    /// Optional embedding model id used by the embedding service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model: Option<String>,
    /// Optional max input texts per embedding batch request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_batch_max_size: Option<usize>,
    /// Optional max concurrent embedding chunks per batch request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_batch_max_concurrency: Option<usize>,
    /// Optional per-attempt timeout (ms) for memory embedding calls.
    ///
    /// This timeout is used by semantic memory recall/store operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_timeout_ms: Option<u64>,
    /// Optional cooldown window (ms) after an embedding timeout.
    ///
    /// During cooldown, memory embedding requests are rejected quickly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_timeout_cooldown_ms: Option<u64>,
    /// Embedding dimension for intent vectors (must match encoder).
    #[serde(default = "memory_defaults::default_embedding_dim")]
    pub embedding_dim: usize,
    /// Table name for episodes.
    #[serde(default = "memory_defaults::default_memory_table")]
    pub table_name: String,
    /// Phase 1 candidate count for two-phase recall.
    #[serde(default = "memory_defaults::default_recall_k1")]
    pub recall_k1: usize,
    /// Phase 2 result count after Q-value reranking.
    #[serde(default = "memory_defaults::default_recall_k2")]
    pub recall_k2: usize,
    /// Q-value weight in reranking (0.0 = semantic only, 1.0 = Q only).
    #[serde(default = "memory_defaults::default_recall_lambda")]
    pub recall_lambda: f32,
    /// Persistence backend mode: auto/local/valkey.
    #[serde(default = "memory_defaults::default_memory_persistence_backend")]
    pub persistence_backend: String,
    /// Optional Valkey URL override injected by runtime builder/tests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_valkey_url: Option<String>,
    /// Key prefix for Valkey-backed memory state.
    #[serde(default = "memory_defaults::default_memory_persistence_key_prefix")]
    pub persistence_key_prefix: String,
    /// Optional strict-startup override for Valkey-backed persistence.
    ///
    /// - `Some(true)`: fail startup when initial Valkey load fails.
    /// - `Some(false)`: continue startup with empty memory on load failure.
    /// - `None`: use backend defaults (strict for Valkey, relaxed for local).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_strict_startup: Option<bool>,
    /// Whether to apply post-turn recall credit updates to recalled episodes.
    #[serde(default = "memory_defaults::default_recall_credit_enabled")]
    pub recall_credit_enabled: bool,
    /// Maximum recalled episodes to receive post-turn credit updates.
    #[serde(default = "memory_defaults::default_recall_credit_max_candidates")]
    pub recall_credit_max_candidates: usize,
    /// Whether to apply periodic memory decay.
    #[serde(default = "memory_defaults::default_decay_enabled")]
    pub decay_enabled: bool,
    /// Apply memory decay every N successful stored turns.
    #[serde(default = "memory_defaults::default_decay_every_turns")]
    pub decay_every_turns: usize,
    /// Decay factor passed to memory store decay routine.
    #[serde(default = "memory_defaults::default_decay_factor")]
    pub decay_factor: f32,
    /// Utility threshold for promote gate decision.
    #[serde(default = "memory_defaults::default_gate_promote_threshold")]
    pub gate_promote_threshold: f32,
    /// Utility threshold for obsolete gate decision.
    #[serde(default = "memory_defaults::default_gate_obsolete_threshold")]
    pub gate_obsolete_threshold: f32,
    /// Minimum usage count required before promote is allowed.
    #[serde(default = "memory_defaults::default_gate_promote_min_usage")]
    pub gate_promote_min_usage: u32,
    /// Minimum usage count required before obsolete is allowed.
    #[serde(default = "memory_defaults::default_gate_obsolete_min_usage")]
    pub gate_obsolete_min_usage: u32,
    /// Failure-rate ceiling for promote gate decision.
    #[serde(default = "memory_defaults::default_gate_promote_failure_rate_ceiling")]
    pub gate_promote_failure_rate_ceiling: f32,
    /// Failure-rate floor for obsolete gate decision.
    #[serde(default = "memory_defaults::default_gate_obsolete_failure_rate_floor")]
    pub gate_obsolete_failure_rate_floor: f32,
    /// Minimum TTL score for promote gate decision.
    #[serde(default = "memory_defaults::default_gate_promote_min_ttl_score")]
    pub gate_promote_min_ttl_score: f32,
    /// Maximum TTL score for obsolete gate decision.
    #[serde(default = "memory_defaults::default_gate_obsolete_max_ttl_score")]
    pub gate_obsolete_max_ttl_score: f32,
    /// Enable Valkey memory stream consumer (`memory.events` -> learning metrics).
    #[serde(default = "memory_defaults::default_stream_consumer_enabled")]
    pub stream_consumer_enabled: bool,
    /// Valkey stream name to consume memory events from.
    #[serde(default = "memory_defaults::default_stream_name")]
    pub stream_name: String,
    /// Consumer group name for memory event stream processing.
    #[serde(default = "memory_defaults::default_stream_consumer_group")]
    pub stream_consumer_group: String,
    /// Consumer name prefix (final consumer name includes pid + timestamp suffix).
    #[serde(default = "memory_defaults::default_stream_consumer_name_prefix")]
    pub stream_consumer_name_prefix: String,
    /// Max events read per XREADGROUP poll.
    #[serde(default = "memory_defaults::default_stream_consumer_batch_size")]
    pub stream_consumer_batch_size: usize,
    /// Block timeout (milliseconds) for XREADGROUP polling.
    #[serde(default = "memory_defaults::default_stream_consumer_block_ms")]
    pub stream_consumer_block_ms: u64,
}

/// Agent config: inference API + MCP server list + optional memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Chat completions endpoint (e.g. `https://api.openai.com/v1/chat/completions` or `LiteLLM`).
    pub inference_url: String,
    /// Model id (e.g. `gpt-4o-mini`, `claude-3-5-sonnet`).
    pub model: String,
    /// API key; if None, read from env `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` depending on URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// MCP servers to connect to (tools from all are merged).
    #[serde(default)]
    pub mcp_servers: Vec<McpServerEntry>,
    /// MCP pool size for concurrent tool calls.
    #[serde(default = "agent_defaults::default_mcp_pool_size")]
    pub mcp_pool_size: usize,
    /// MCP handshake timeout per connect attempt, in seconds.
    #[serde(default = "agent_defaults::default_mcp_handshake_timeout_secs")]
    pub mcp_handshake_timeout_secs: u64,
    /// MCP connect retries before failing startup.
    #[serde(default = "agent_defaults::default_mcp_connect_retries")]
    pub mcp_connect_retries: u32,
    /// If true, MCP startup/connect failures abort agent startup.
    /// If false, agent starts without MCP and degrades tool execution gracefully.
    #[serde(default = "agent_defaults::default_mcp_strict_startup")]
    pub mcp_strict_startup: bool,
    /// Initial backoff between MCP connect retries, in milliseconds.
    #[serde(default = "agent_defaults::default_mcp_connect_retry_backoff_ms")]
    pub mcp_connect_retry_backoff_ms: u64,
    /// MCP tool call timeout, in seconds.
    #[serde(default = "agent_defaults::default_mcp_tool_timeout_secs")]
    pub mcp_tool_timeout_secs: u64,
    /// MCP tools/list snapshot cache TTL (milliseconds) on the Rust client side.
    #[serde(default = "agent_defaults::default_mcp_list_tools_cache_ttl_ms")]
    pub mcp_list_tools_cache_ttl_ms: u64,
    /// Max tool-call rounds per user turn (avoid infinite loops).
    #[serde(default = "agent_defaults::default_max_tool_rounds")]
    pub max_tool_rounds: u32,
    /// Optional omni-memory config (two-phase recall + `store_episode`). None = memory disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryConfig>,
    /// If set, use omni-window (ring buffer) for session history with this max turns; context for LLM is built from window. None = use in-memory `SessionStore` (unbounded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_max_turns: Option<usize>,
    /// When window turn count >= this, consolidate oldest segment into omni-memory. None = consolidation disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consolidation_threshold_turns: Option<usize>,
    /// Number of oldest turns to drain per consolidation (when threshold exceeded). Ignored if consolidation disabled.
    #[serde(default = "agent_defaults::default_consolidation_take_turns")]
    pub consolidation_take_turns: usize,
    /// If true, store consolidated memory episodes in background task.
    #[serde(default = "agent_defaults::default_consolidation_async")]
    pub consolidation_async: bool,
    /// Optional token budget for prompt context packing. None = no token-budget pruning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_budget_tokens: Option<usize>,
    /// Reserved tokens in context budget to avoid packing right at hard limit.
    #[serde(default = "agent_defaults::default_context_budget_reserve_tokens")]
    pub context_budget_reserve_tokens: usize,
    /// Strategy for deciding which context classes are retained first under tight budget.
    #[serde(default)]
    pub context_budget_strategy: ContextBudgetStrategy,
    /// Maximum number of compacted summary segments injected into prompt context.
    #[serde(default = "agent_defaults::default_summary_max_segments")]
    pub summary_max_segments: usize,
    /// Maximum chars kept per compacted summary segment.
    #[serde(default = "agent_defaults::default_summary_max_chars")]
    pub summary_max_chars: usize,
}

/// Prompt context budget retention strategy under tight token constraints.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextBudgetStrategy {
    /// Keep recent dialogue turns ahead of compacted summary segments.
    #[default]
    RecentFirst,
    /// Keep compacted summary segments ahead of older dialogue turns.
    SummaryFirst,
}

impl ContextBudgetStrategy {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RecentFirst => "recent_first",
            Self::SummaryFirst => "summary_first",
        }
    }
}
