//! Swarm pulse telemetry contracts and Valkey emitter.

mod events;
mod traits;
mod valkey;

pub use events::{
    ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase, SwarmEvent, unix_millis_now,
};
pub use traits::{NoopPulseEmitter, PulseEmitter};
pub use valkey::ValkeyPulseEmitter;
