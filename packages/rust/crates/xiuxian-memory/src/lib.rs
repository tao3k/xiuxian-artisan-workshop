//! xiuxian-memory: `MemRL` self-evolving memory system.

pub mod core;

pub use core::learner::MemRLCortex;
pub use core::types::{MemoryAction, MemoryState};
