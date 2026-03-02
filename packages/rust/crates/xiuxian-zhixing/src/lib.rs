//! xiuxian-zhixing - The 'Unity of Knowledge and Action' logic layer.

/// Compile-time embedded resource tree rooted at `xiuxian-zhixing/resources`.
pub static RESOURCES: ::include_dir::Dir<'_> =
    ::include_dir::include_dir!("$CARGO_MANIFEST_DIR/resources");

/// Action compiler runtime (Cognitive-Execution Decoupling backend).
pub mod action_compiler;
/// Agenda domain models and helpers.
pub mod agenda;
/// Error types for Zhixing orchestration.
pub mod error;
/// Heyi orchestration primitives bridging knowledge and action.
pub mod heyi;
/// Journal domain models.
pub mod journal;
/// Storage backends for notebook/task persistence.
pub mod storage;

pub use agenda::AgendaEntry;
pub use error::{Error, Result};
pub use heyi::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_RECIPIENT, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED,
    DueReminderRecord, ReminderQueueSettings, ReminderQueueStore, ReminderSignal, ZhixingHeyi,
};
pub use journal::JournalEntry;
