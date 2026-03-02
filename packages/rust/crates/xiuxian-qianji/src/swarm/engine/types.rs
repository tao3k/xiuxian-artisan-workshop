use crate::error::QianjiError;
use crate::telemetry::PulseEmitter;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) type WorkerJoinSet = tokio::task::JoinSet<Result<SwarmAgentReport, QianjiError>>;

#[derive(Debug, Clone)]
pub(super) struct WorkerRuntimeConfig {
    pub(super) session_id: String,
    pub(super) redis_url: Option<String>,
    pub(super) cluster_id: Option<String>,
    pub(super) remote_enabled: bool,
    pub(super) poll_interval_ms: u64,
    pub(super) allow_local_affinity_proxy: bool,
    pub(super) pulse_emitter: Option<Arc<dyn PulseEmitter>>,
}

pub(super) fn generate_swarm_session_id() -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random_suffix: u64 = rand::random();
    format!("swarm_{now_ms}_{random_suffix:x}")
}

/// Runtime identity and scheduler hints for one swarm worker.
#[derive(Debug, Clone)]
pub struct SwarmAgentConfig {
    /// Stable logical agent id (used for consensus vote identity).
    pub agent_id: String,
    /// Optional role class for node routing (for example `student`/`steward`).
    pub role_class: Option<String>,
    /// Vote weight used by distributed consensus policy.
    pub weight: f32,
    /// Session window size for this worker.
    pub window_size: usize,
}

impl SwarmAgentConfig {
    /// Creates a new agent profile with defaults.
    #[must_use]
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            role_class: None,
            weight: 1.0,
            window_size: 1000,
        }
    }
}

/// One worker run result from swarm orchestration.
#[derive(Debug, Clone)]
pub struct SwarmAgentReport {
    /// Worker identity.
    pub agent_id: String,
    /// Optional worker role class.
    pub role_class: Option<String>,
    /// Whether this worker finished successfully.
    pub success: bool,
    /// Final workflow context for this worker on success.
    pub context: Option<serde_json::Value>,
    /// Error message on failure.
    pub error: Option<String>,
    /// Number of turns kept in the local session window.
    pub window_turns: u64,
    /// Number of tool calls tracked in the local session window.
    pub window_tool_calls: u64,
}

/// Swarm execution options.
#[derive(Debug, Clone)]
pub struct SwarmExecutionOptions {
    /// Shared session id for all workers. Auto-generated when not provided.
    pub session_id: Option<String>,
    /// Optional Valkey URL used for checkpoint/consensus synchronization.
    pub redis_url: Option<String>,
    /// Optional cluster id used by remote possession routing.
    pub cluster_id: Option<String>,
    /// Enables background remote-possession responder loop for each role worker.
    pub enable_remote_possession: bool,
    /// Poll interval for background remote-possession responder.
    pub possession_poll_interval_ms: u64,
    /// Allows manager/auditor workers to proxy missing roles when no global candidate exists.
    pub allow_local_affinity_proxy: bool,
    /// Optional pulse telemetry emitter used for non-blocking observability events.
    pub pulse_emitter: Option<Arc<dyn PulseEmitter>>,
}

impl Default for SwarmExecutionOptions {
    fn default() -> Self {
        Self {
            session_id: None,
            redis_url: None,
            cluster_id: None,
            enable_remote_possession: false,
            possession_poll_interval_ms: 500,
            allow_local_affinity_proxy: true,
            pulse_emitter: None,
        }
    }
}

/// Final report for one swarm execution.
#[derive(Debug, Clone)]
pub struct SwarmExecutionReport {
    /// Shared session id used by all workers.
    pub session_id: String,
    /// Selected final context (first successful worker output).
    pub final_context: serde_json::Value,
    /// Per-agent run reports.
    pub workers: Vec<SwarmAgentReport>,
}
