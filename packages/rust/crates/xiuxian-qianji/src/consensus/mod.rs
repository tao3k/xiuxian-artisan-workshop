//! Distributed consensus module.
//!
//! `mod.rs` intentionally contains declarations and re-exports only.

mod manager;
mod models;
mod thresholds;

pub use manager::ConsensusManager;
pub use models::{AgentIdentity, AgentVote, ConsensusMode, ConsensusPolicy, ConsensusResult};
