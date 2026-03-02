use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default Valkey channel used for swarm pulse telemetry.
pub const DEFAULT_PULSE_CHANNEL: &str = "xiuxian:swarm:pulse";

/// Node transition stage in scheduler execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeTransitionPhase {
    Entering,
    Exiting,
    Failed,
}

/// Consensus lifecycle status used by pulse telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusStatus {
    Pending,
    Agreed,
    Failed,
}

/// Typed swarm telemetry event envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SwarmEvent {
    /// Lightweight worker heartbeat used for liveness/cluster monitoring.
    SwarmHeartbeat {
        session_id: Option<String>,
        cluster_id: Option<String>,
        agent_id: Option<String>,
        role_class: Option<String>,
        cpu_percent: Option<f32>,
        memory_bytes: Option<u64>,
        timestamp_ms: u64,
    },
    /// Scheduler node lifecycle transition.
    NodeTransition {
        session_id: Option<String>,
        agent_id: Option<String>,
        role_class: Option<String>,
        node_id: String,
        phase: NodeTransitionPhase,
        timestamp_ms: u64,
    },
    /// Consensus state signal for observability consumers.
    ConsensusSpike {
        session_id: String,
        node_id: String,
        status: ConsensusStatus,
        progress: Option<f32>,
        target: Option<f32>,
        timestamp_ms: u64,
    },
    /// Event fired when one manifestation artifact is produced.
    EvolutionBirth {
        session_id: Option<String>,
        role_id: Option<String>,
        manifestation_path: String,
        timestamp_ms: u64,
    },
    /// Affinity failover warning when local proxy delegation is activated.
    AffinityAlert {
        session_id: Option<String>,
        node_id: String,
        required_role: String,
        proxy_agent_id: Option<String>,
        proxy_role: Option<String>,
        timestamp_ms: u64,
    },
}

/// Returns current UNIX timestamp in milliseconds.
#[must_use]
pub fn unix_millis_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
