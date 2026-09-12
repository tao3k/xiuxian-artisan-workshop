//! Evidence about a completed, possibly non-atomic hot-state collection.

use serde::{Deserialize, Serialize};

/// Missing records actually witnessed during collection, not a completeness proof.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotStateMissingData {
    /// Pending step index entries with no payload.
    pub pending_step_payloads: usize,
    /// Leased step index entries with no payload.
    pub leased_step_payloads: usize,
    /// Leased step payloads with no lease record.
    pub step_leases: usize,
    /// Pending activity index entries with no payload.
    pub pending_activity_payloads: usize,
    /// Leased activity index entries with no payload.
    pub leased_activity_payloads: usize,
    /// Leased activity payloads with no lease record.
    pub activity_leases: usize,
    /// Enumerated heartbeat keys that vanished before payload read.
    pub heartbeats: usize,
}

/// Request-wide admission receipt; excludes protocol handshake and wire decoding.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotStateReadUsage {
    /// Actual application command submissions, including transport retries.
    pub commands: usize,
    /// Enumerated queue entries and distinct heartbeat keys combined.
    pub items: usize,
    /// Cumulative admitted raw key, payload, and lease bytes.
    pub bytes: usize,
}

/// Collection evidence. Wall-clock timestamps may reflect clock adjustments;
/// use the snapshot's monotonic duration for latency. No atomicity is implied.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotStateObservation {
    /// Wall-clock start, distinct from the caller's expiry reference.
    pub started_at_ms: u64,
    /// Wall-clock finish.
    pub finished_at_ms: u64,
    /// Missing components witnessed during the collection.
    pub missing: HotStateMissingData,
    /// Shared read admission usage.
    pub usage: HotStateReadUsage,
}
