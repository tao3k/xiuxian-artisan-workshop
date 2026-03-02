//! Core agent logic and loop implementation.

mod admission;
mod bootstrap;
mod consolidation;
mod context_budget;
mod context_budget_state;
mod embedding_runtime;
mod feedback;
mod injection;
pub(crate) mod logging;
mod mcp;
mod mcp_pool_state;
mod mcp_startup;
mod memory;
mod memory_recall;
mod memory_recall_feedback;
mod memory_recall_metrics;
mod memory_recall_state;
mod memory_state;
mod memory_stream_consumer;
pub mod native_tools;
pub mod notification;
mod omega;
mod persistence;
mod reflection;
mod reflection_runtime_state;
mod session_context;
mod system_prompt_injection_state;
mod turn_execution;
mod turn_support;
mod zhenfa;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::RwLock;

use omni_memory::EpisodeStore;
use xiuxian_llm::embedding::runtime::EmbeddingRuntime;
use xiuxian_qianhuan::{HotReloadDriver, ManifestationManager};
pub use xiuxian_zhixing::ZhixingHeyi;

use crate::config::AgentConfig;
use crate::embedding::EmbeddingClient;
use crate::llm::LlmClient;
use crate::session::{BoundedSessionStore, SessionStore};
use memory_state::{MemoryStateBackend, MemoryStateLoadStatus};
pub use native_tools::NativeToolRegistry;
use reflection::PolicyHintDirective;

pub(crate) use admission::DownstreamAdmissionRuntimeSnapshot;
pub use bootstrap::ServiceMountRecord;
pub use consolidation::summarise_drained_turns;
pub use context_budget::prune_messages_for_token_budget;
pub use context_budget_state::{SessionContextBudgetClassSnapshot, SessionContextBudgetSnapshot};
pub use memory_recall_metrics::{MemoryRecallLatencyBucketsSnapshot, MemoryRecallMetricsSnapshot};
pub use memory_recall_state::{SessionMemoryRecallDecision, SessionMemoryRecallSnapshot};
pub use memory_state::MemoryRuntimeStatusSnapshot;
pub use session_context::{
    SessionContextMode, SessionContextSnapshotInfo, SessionContextStats, SessionContextWindowInfo,
};

/// Explicit session-level recall feedback direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRecallFeedbackDirection {
    /// Feedback direction up.
    Up,
    /// Feedback direction down.
    Down,
}

/// Result of applying explicit session-level recall feedback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionRecallFeedbackUpdate {
    /// Bias before the update.
    pub previous_bias: f32,
    /// Bias after the update.
    pub updated_bias: f32,
    /// Direction applied.
    pub direction: SessionRecallFeedbackDirection,
}

/// Agent: config + session store (or bounded session) + LLM client + optional MCP pool + optional memory.
pub struct Agent {
    config: AgentConfig,
    session: SessionStore,
    /// Idle-time threshold for auto reset policy (milliseconds). None disables idle reset.
    session_reset_idle_timeout_ms: Option<u64>,
    /// Last observed activity timestamp by session scope.
    session_last_activity_unix_ms: Arc<RwLock<HashMap<String, u64>>>,
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
    /// Embedding runtime policy guard (timeout/cooldown/repair).
    embedding_runtime: Option<Arc<EmbeddingRuntime>>,
    /// Most recent context-budget report by logical session id.
    context_budget_snapshots: Arc<RwLock<HashMap<String, SessionContextBudgetSnapshot>>>,
    /// Process-level memory recall metrics snapshot (for diagnostics dashboards).
    memory_recall_metrics: Arc<RwLock<memory_recall_metrics::MemoryRecallMetricsState>>,
    /// Runtime manifestation manager (owns prompt injection cache/state).
    manifestation_manager: Option<Arc<ManifestationManager>>,
    /// One-shot next-turn policy hints derived from reflection lifecycle.
    reflection_policy_hints: Arc<RwLock<HashMap<String, PolicyHintDirective>>>,
    /// Counter used by periodic memory decay policy.
    memory_decay_turn_counter: Arc<AtomicU64>,
    downstream_admission_policy: admission::DownstreamAdmissionPolicy,
    downstream_admission_metrics: admission::DownstreamAdmissionMetrics,
    llm: LlmClient,
    mcp: Option<crate::mcp::McpClientPool>,
    heyi: Option<Arc<ZhixingHeyi>>,
    native_tools: Arc<NativeToolRegistry>,
    zhenfa_tools: Option<Arc<zhenfa::ZhenfaToolBridge>>,
    memory_stream_consumer_task: Option<tokio::task::JoinHandle<()>>,
    _hot_reload_driver: Option<HotReloadDriver>,
    /// Bootstrap-time service mount records for runtime diagnostics and reporting.
    service_mount_records: Arc<RwLock<Vec<ServiceMountRecord>>>,
}

impl Drop for Agent {
    fn drop(&mut self) {
        if let Some(task) = self.memory_stream_consumer_task.take() {
            task.abort();
        }
    }
}

impl Agent {
    /// Returns bootstrap-time mount records for all service wiring.
    pub async fn service_mount_records(&self) -> Vec<ServiceMountRecord> {
        self.service_mount_records.read().await.clone()
    }

    /// Returns the internal `ZhixingHeyi` orchestrator if initialized.
    #[must_use]
    pub fn get_heyi(&self) -> Option<Arc<ZhixingHeyi>> {
        self.heyi.clone()
    }
}
