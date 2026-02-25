mod admin;
mod injection;

use super::shared::{
    SessionPartitionModeToken, is_reset_context_command as is_reset_context_command_shared,
    is_stop_command as is_stop_command_shared, parse_resume_context_command as parse_resume_shared,
    parse_session_context_budget_command as parse_session_budget_shared,
    parse_session_context_memory_command as parse_session_memory_shared,
    parse_session_context_status_command as parse_session_status_shared,
    parse_session_feedback_command as parse_session_feedback_shared,
    parse_session_partition_command as parse_session_partition_shared,
    parse_session_partition_mode_token as parse_partition_mode_token,
};

pub use admin::parse_session_admin_command;
pub use injection::parse_session_injection_command;

pub type ResumeContextCommand = super::shared::ResumeCommand;
pub type SessionFeedbackDirection = super::shared::FeedbackDirection;
pub type SessionFeedbackCommand = super::shared::SessionFeedbackCommand;
pub type SessionOutputFormat = super::shared::OutputFormat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionInjectionAction {
    Status,
    Clear,
    SetXml(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInjectionCommand {
    pub action: SessionInjectionAction,
    pub format: SessionOutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionAdminAction {
    List,
    Set(Vec<String>),
    Add(Vec<String>),
    Remove(Vec<String>),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionAdminCommand {
    pub action: SessionAdminAction,
    pub format: SessionOutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPartitionMode {
    Chat,
    ChatUser,
    User,
    ChatThreadUser,
}

impl SessionPartitionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::ChatUser => "chat_user",
            Self::User => "user",
            Self::ChatThreadUser => "chat_thread_user",
        }
    }
}

pub type SessionPartitionCommand = super::shared::SessionPartitionCommand<SessionPartitionMode>;

/// Parse session status command and return output format.
pub fn parse_session_context_status_command(input: &str) -> Option<SessionOutputFormat> {
    parse_session_status_shared(input)
}

/// Parse session budget command and return output format.
pub fn parse_session_context_budget_command(input: &str) -> Option<SessionOutputFormat> {
    parse_session_budget_shared(input)
}

/// Parse session memory command and return output format.
pub fn parse_session_context_memory_command(input: &str) -> Option<SessionOutputFormat> {
    parse_session_memory_shared(input)
}

/// Parse session partition command:
/// - `/session partition` (status)
/// - `/session partition json`
/// - `/session partition on|off`
/// - `/session partition chat|chat_user|user|chat_thread_user [json]`
pub fn parse_session_partition_command(input: &str) -> Option<SessionPartitionCommand> {
    parse_session_partition_shared(input, parse_session_partition_mode)
}

/// Parse session recall-feedback command:
/// - `/session feedback up|down [json]`
/// - `/window feedback up|down [json]`
/// - `/context feedback up|down [json]`
/// - `/feedback up|down [json]`
pub fn parse_session_feedback_command(input: &str) -> Option<SessionFeedbackCommand> {
    parse_session_feedback_shared(input)
}

/// Parse `/reset`, `/clear`, `reset`, or `clear`.
pub fn is_reset_context_command(input: &str) -> bool {
    is_reset_context_command_shared(input)
}

/// Parse `/stop`, `/cancel`, `stop`, `cancel`, or `interrupt`.
pub fn is_stop_command(input: &str) -> bool {
    is_stop_command_shared(input)
}

/// Parse `/resume` or `resume`, with optional `/resume status`.
pub fn parse_resume_context_command(input: &str) -> Option<ResumeContextCommand> {
    parse_resume_shared(input)
}

fn parse_session_partition_mode(raw: &str) -> Option<SessionPartitionMode> {
    let token = parse_partition_mode_token(raw)?;
    match token {
        SessionPartitionModeToken::Chat => Some(SessionPartitionMode::Chat),
        SessionPartitionModeToken::ChatUser => Some(SessionPartitionMode::ChatUser),
        SessionPartitionModeToken::User => Some(SessionPartitionMode::User),
        SessionPartitionModeToken::ChatThreadUser => Some(SessionPartitionMode::ChatThreadUser),
        SessionPartitionModeToken::GuildChannelUser
        | SessionPartitionModeToken::Channel
        | SessionPartitionModeToken::GuildUser => None,
    }
}
