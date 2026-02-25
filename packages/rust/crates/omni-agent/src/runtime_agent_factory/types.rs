use omni_agent::{ContextBudgetStrategy, MemoryConfig};
use xiuxian_llm::embedding::backend::EmbeddingBackendKind;

pub(super) type RuntimeEmbeddingBackendMode = EmbeddingBackendKind;

pub(super) struct McpRuntimeOptions {
    pub(super) pool_size: usize,
    pub(super) handshake_timeout_secs: u64,
    pub(super) connect_retries: u32,
    pub(super) strict_startup: bool,
    pub(super) connect_retry_backoff_ms: u64,
    pub(super) tool_timeout_secs: u64,
    pub(super) list_tools_cache_ttl_ms: u64,
}

pub(super) struct SessionRuntimeOptions {
    pub(super) max_tool_rounds: u32,
    pub(super) window_max_turns: Option<usize>,
    pub(super) consolidation_threshold_turns: Option<usize>,
    pub(super) consolidation_take_turns: usize,
    pub(super) consolidation_async: bool,
    pub(super) context_budget_tokens: Option<usize>,
    pub(super) context_budget_reserve_tokens: usize,
    pub(super) context_budget_strategy: ContextBudgetStrategy,
    pub(super) summary_max_segments: usize,
    pub(super) summary_max_chars: usize,
}

pub(super) struct MemoryRuntimeOptions {
    pub(super) config: MemoryConfig,
    pub(super) embedding_backend_mode: RuntimeEmbeddingBackendMode,
}
