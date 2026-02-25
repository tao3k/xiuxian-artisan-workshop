//! xiuxian-zhixing (修仙-知行合一)
//! 
//! This crate implements an AI-driven Agenda and Journaling system,
//! integrating actionable tasks (Xing/行) with reflected knowledge (Zhi/知).
//! It is inspired by Wang Yangming's "Unity of Knowledge and Action".

/// Agenda and task management logic.
pub mod agenda;
/// Daily reflection and journal logic.
pub mod journal;
/// Orchestration between Zhi (Knowledge) and Xing (Action).
pub mod heyi;
/// Interfaces for LLM and external system interaction.
pub mod interface;
/// Alchemy logic for transforming raw text into structured actions.
pub mod alchemist;

pub use agenda::AgendaEntry;
pub use journal::JournalEntry;
pub use heyi::ZhixingHeyi;
pub use alchemist::Alchemist;

/// Base error type for xiuxian-zhixing.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Errors related to parsing or logic consistency.
    #[error("Logic error: {0}")]
    Logic(String),
    /// Errors related to parsing context.
    #[error("Parsing error: {0}")]
    Parsing(String),
    /// Generic internal errors.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for xiuxian-zhixing operations.
pub type Result<T> = std::result::Result<T, Error>;
