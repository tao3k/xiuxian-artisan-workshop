mod admin;
mod injection;
mod partition;

pub(super) use crate::channels::managed_runtime::parsing::{
    FeedbackDirection, ResumeCommand, SessionFeedbackCommand,
    SessionPartitionCommand as SharedSessionPartitionCommand,
};
use crate::channels::managed_runtime::parsing::{
    OutputFormat, parse_background_prompt, parse_help_command, parse_job_status_command,
    parse_jobs_summary_command, parse_resume_context_command, parse_session_context_budget_command,
    parse_session_context_memory_command, parse_session_context_status_command,
    parse_session_feedback_command,
};

use super::super::super::session_partition::DiscordSessionPartition;

use admin::parse_session_admin_command;
use injection::parse_session_injection_command;
use partition::parse_session_partition_command;

pub(super) type CommandOutputFormat = OutputFormat;
type SessionPartitionMode = DiscordSessionPartition;
pub(super) type SessionPartitionCommand = SharedSessionPartitionCommand<SessionPartitionMode>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SessionInjectionAction {
    Status,
    Clear,
    SetXml(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SessionInjectionCommand {
    pub(super) action: SessionInjectionAction,
    pub(super) format: CommandOutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SessionAdminAction {
    List,
    Set(Vec<String>),
    Add(Vec<String>),
    Remove(Vec<String>),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SessionAdminCommand {
    pub(super) action: SessionAdminAction,
    pub(super) format: CommandOutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ManagedCommand {
    Help(CommandOutputFormat),
    Reset,
    Resume(ResumeCommand),
    SessionStatus(CommandOutputFormat),
    SessionBudget(CommandOutputFormat),
    SessionMemory(CommandOutputFormat),
    SessionFeedback(SessionFeedbackCommand),
    SessionPartition(SessionPartitionCommand),
    SessionAdmin(SessionAdminCommand),
    SessionInjection(SessionInjectionCommand),
    JobStatus {
        job_id: String,
        format: CommandOutputFormat,
    },
    JobsSummary(CommandOutputFormat),
    BackgroundSubmit(String),
}

pub(super) fn parse_managed_command(input: &str) -> Option<ManagedCommand> {
    if let Some(format) = parse_help_command(input) {
        return Some(ManagedCommand::Help(format));
    }
    if crate::channels::managed_runtime::parsing::is_reset_context_command(input) {
        return Some(ManagedCommand::Reset);
    }
    if let Some(resume) = parse_resume_context_command(input) {
        return Some(ManagedCommand::Resume(resume));
    }
    if let Some(command) = parse_session_admin_command(input) {
        return Some(ManagedCommand::SessionAdmin(command));
    }
    if let Some(command) = parse_session_injection_command(input) {
        return Some(ManagedCommand::SessionInjection(command));
    }
    if let Some(command) = parse_session_partition_command(input) {
        return Some(ManagedCommand::SessionPartition(command));
    }
    if let Some(format) = parse_session_context_status_command(input) {
        return Some(ManagedCommand::SessionStatus(format));
    }
    if let Some(format) = parse_session_context_budget_command(input) {
        return Some(ManagedCommand::SessionBudget(format));
    }
    if let Some(format) = parse_session_context_memory_command(input) {
        return Some(ManagedCommand::SessionMemory(format));
    }
    if let Some(command) = parse_session_feedback_command(input) {
        return Some(ManagedCommand::SessionFeedback(command));
    }
    if let Some(command) = parse_job_status_command(input) {
        return Some(ManagedCommand::JobStatus {
            job_id: command.job_id,
            format: command.format,
        });
    }
    if let Some(format) = parse_jobs_summary_command(input) {
        return Some(ManagedCommand::JobsSummary(format));
    }
    if let Some(prompt) = parse_background_prompt(input) {
        return Some(ManagedCommand::BackgroundSubmit(prompt));
    }
    None
}
