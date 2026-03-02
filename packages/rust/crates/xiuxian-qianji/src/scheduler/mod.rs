//! Asynchronous synaptic-flow scheduler.

/// Valkey checkpointing integration for persisting workflow state.
pub mod checkpoint;

/// Core execution loop and scheduler logic.
pub mod core;

/// Scheduler execution identity for role-aware distributed routing.
pub mod identity;
/// Scheduler execution policy and role availability probing contracts.
pub mod policy;

/// Context preflight and semantic placeholder resolution.
pub(crate) mod preflight;

/// Graph topological tracking and scheduling state.
pub mod state;

pub use self::core::QianjiScheduler;
pub use self::identity::SchedulerAgentIdentity;
pub use self::policy::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
