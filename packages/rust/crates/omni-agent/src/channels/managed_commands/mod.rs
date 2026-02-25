//! Shared managed-command classification and slash scope constants.
//!
//! These commands are platform-facing operational commands (session/job/background control)
//! handled outside generic LLM conversation flow.

mod control_detection;
mod input_normalization;
mod slash_detection;
mod types;

pub(crate) use control_detection::detect_managed_control_command;
pub(crate) use slash_detection::detect_managed_slash_command;
pub(crate) use types::{
    ManagedControlCommand, ManagedSlashCommand, SLASH_SCOPE_BACKGROUND_SUBMIT,
    SLASH_SCOPE_JOB_STATUS, SLASH_SCOPE_JOBS_SUMMARY, SLASH_SCOPE_SESSION_BUDGET,
    SLASH_SCOPE_SESSION_FEEDBACK, SLASH_SCOPE_SESSION_MEMORY, SLASH_SCOPE_SESSION_STATUS,
};
