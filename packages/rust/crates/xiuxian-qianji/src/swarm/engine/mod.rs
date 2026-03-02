//! Inner-Rust swarm orchestration with per-thread window isolation.

mod orchestrator;
mod types;
mod worker;

pub use orchestrator::SwarmEngine;
pub use types::{SwarmAgentConfig, SwarmAgentReport, SwarmExecutionOptions, SwarmExecutionReport};
