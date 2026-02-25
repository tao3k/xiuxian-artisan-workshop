//! One-turn agent loop: user message -> LLM (+ optional tools) -> `tool_calls` -> MCP tools/call -> repeat.

mod admission;
mod bootstrap;
mod consolidation;
mod context_budget;
mod context_budget_state;
mod embedding_dimension;
mod embedding_runtime;
mod feedback;
mod graph;
mod graph_bridge;
mod injection;
pub(crate) mod logging;
mod mcp;
mod mcp_pool_state;
mod mcp_startup;
mod memory;
mod memory_recall;
mod memory_recall_feedback;
mod memory_recall_feedback_state;
mod memory_recall_metrics;
mod memory_recall_state;
mod memory_state;
mod memory_stream_consumer;
mod omega;
mod persistence;
mod reflection;
mod reflection_runtime_state;
mod session_context;
mod system_prompt_injection_state;
mod turn_execution;
mod turn_support;

use anyhow::Result;
use omni_tokenizer::count_tokens;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use omni_memory::EpisodeStore;
use xiuxian_qianhuan::InjectionPolicy;
use xiuxian_zhixing::ZhixingHeyi;

use crate::config::AgentConfig;
use crate::contracts::{OmegaDecision, OmegaFallbackPolicy, OmegaRoute};
use crate::embedding::EmbeddingClient;
use crate::llm::LlmClient;
use crate::observability::SessionEvent;
use crate::session::{BoundedSessionStore, ChatMessage, SessionStore, SessionSummarySegment};
use crate::shortcuts::parse_react_shortcut;
use embedding_dimension::{
    EMBEDDING_SOURCE_EMBEDDING, EMBEDDING_SOURCE_EMBEDDING_REPAIRED, repair_embedding_dimension,
};
use embedding_runtime::EMBEDDING_SOURCE_UNAVAILABLE;
use memory::{RecalledEpisodeCandidate, apply_recall_credit, select_recall_credit_candidates};
use memory_recall::{
    MEMORY_RECALL_MESSAGE_NAME, MemoryRecallInput, build_memory_context_message,
    estimate_messages_tokens, filter_recalled_episodes, plan_memory_recall,
};
use memory_recall_feedback::{
    RECALL_FEEDBACK_SOURCE_COMMAND, RecallOutcome, ToolExecutionSummary, apply_feedback_to_plan,
    resolve_feedback_outcome, update_feedback_bias,
};
use memory_state::{MemoryStateBackend, MemoryStateLoadStatus};
use reflection::PolicyHintDirective;
use system_prompt_injection_state::SYSTEM_PROMPT_INJECTION_CONTEXT_MESSAGE_NAME;

const DEFAULT_MEMORY_EMBED_TIMEOUT: Duration = Duration::from_secs(3);
const DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN: Duration = Duration::from_secs(20);
const MIN_MEMORY_EMBED_TIMEOUT_MS: u64 = 100;
const MAX_MEMORY_EMBED_TIMEOUT_MS: u64 = 60_000;
const MAX_MEMORY_EMBED_COOLDOWN_MS: u64 = 300_000;

pub(crate) use admission::DownstreamAdmissionRuntimeSnapshot;
pub use consolidation::summarise_drained_turns;
pub use context_budget::prune_messages_for_token_budget;
pub use context_budget_state::{SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot};
pub use graph_bridge::{GraphBridgeRequest, GraphBridgeResult, validate_graph_bridge_request};
pub use memory_recall_metrics::{MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot};
pub use memory_recall_state::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};
pub use memory_state::MemoryRuntimeStatusSnapshot;
pub use session_context::{
    SessionContextMode, SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
};
pub use system_prompt_injection_state::SessionSystemPromptInjectionSnapshot;

/// Explicit session-level recall feedback direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRecallFeedbackDirection {
    Up,
    Down,
}

/// Result of applying explicit session-level recall feedback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionRecallFeedbackUpdate {
    pub previous_bias: f32,
    pub updated_bias: f32,
    pub direction: SessionRecallFeedbackDirection,
}

/// Agent: config + session store (or bounded session) + LLM client + optional MCP pool + optional memory.
pub struct Agent {
    config: AgentConfig,
    session: SessionStore,
    /// When set, session history is bounded; context built from recent turns.
    bounded_session: Option<BoundedSessionStore>,
    /// When set (and window enabled), consolidation stores episodes into omni-memory.
    memory_store: Option<Arc<EpisodeStore>>,
    /// Memory persistence backend for episode/Q state snapshots.
    memory_state_backend: Option<Arc<MemoryStateBackend>>,
    /// Startup load status for memory state persistence.
    memory_state_load_status: MemoryStateLoadStatus,
    /// Embedding client for semantic memory recall/store.
    embedding_client: Option<EmbeddingClient>,
    /// Most recent context-budget report by logical session id.
    context_budget_snapshots: Arc<RwLock<HashMap<String, SessionContextBudgetSnapshot>>>,
    /// Process-level memory recall metrics snapshot (for diagnostics dashboards).
    memory_recall_metrics: Arc<RwLock<memory_recall_metrics::MemoryRecallMetricsState>>,
    /// Session-level recall feedback bias (-1: broaden recall, +1: tighten recall).
    memory_recall_feedback: Arc<RwLock<HashMap<String, f32>>>,
    /// Session-level injected system prompt window (XML Q&A).
    system_prompt_injection: Arc<RwLock<HashMap<String, SessionSystemPromptInjectionSnapshot>>>,
    /// One-shot next-turn policy hints derived from reflection lifecycle.
    reflection_policy_hints: Arc<RwLock<HashMap<String, PolicyHintDirective>>>,
    /// Counter used by periodic memory decay policy.
    memory_decay_turn_counter: Arc<AtomicU64>,
    /// Per-attempt timeout for memory embedding requests.
    memory_embed_timeout: Duration,
    /// Cooldown window after an embedding timeout to avoid repeated long waits.
    memory_embed_timeout_cooldown: Duration,
    /// Unix timestamp millis until which embedding calls are rejected by cooldown policy.
    memory_embed_timeout_cooldown_until_ms: AtomicU64,
    downstream_admission_policy: admission::DownstreamAdmissionPolicy,
    downstream_admission_metrics: admission::DownstreamAdmissionMetrics,
    llm: LlmClient,
    mcp: Option<crate::mcp::McpClientPool>,
    heyi: Option<Arc<ZhixingHeyi>>,
    memory_stream_consumer_task: Option<tokio::task::JoinHandle<()>>,
}

impl Drop for Agent {
    fn drop(&mut self) {
        if let Some(task) = self.memory_stream_consumer_task.take() {
            task.abort();
        }
    }
}
