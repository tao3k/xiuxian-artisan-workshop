pub(crate) const SLASH_SCOPE_SESSION_STATUS: &str = "session.status";
pub(crate) const SLASH_SCOPE_SESSION_BUDGET: &str = "session.budget";
pub(crate) const SLASH_SCOPE_SESSION_MEMORY: &str = "session.memory";
pub(crate) const SLASH_SCOPE_SESSION_FEEDBACK: &str = "session.feedback";
pub(crate) const SLASH_SCOPE_JOB_STATUS: &str = "job.status";
pub(crate) const SLASH_SCOPE_JOBS_SUMMARY: &str = "jobs.summary";
pub(crate) const SLASH_SCOPE_BACKGROUND_SUBMIT: &str = "background.submit";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagedSlashCommand {
    SessionStatus,
    SessionBudget,
    SessionMemory,
    SessionFeedback,
    JobStatus,
    JobsSummary,
    BackgroundSubmit,
}

impl ManagedSlashCommand {
    pub(crate) const fn scope(self) -> &'static str {
        match self {
            Self::SessionStatus => SLASH_SCOPE_SESSION_STATUS,
            Self::SessionBudget => SLASH_SCOPE_SESSION_BUDGET,
            Self::SessionMemory => SLASH_SCOPE_SESSION_MEMORY,
            Self::SessionFeedback => SLASH_SCOPE_SESSION_FEEDBACK,
            Self::JobStatus => SLASH_SCOPE_JOB_STATUS,
            Self::JobsSummary => SLASH_SCOPE_JOBS_SUMMARY,
            Self::BackgroundSubmit => SLASH_SCOPE_BACKGROUND_SUBMIT,
        }
    }

    pub(crate) const fn canonical_command(self) -> &'static str {
        match self {
            Self::SessionStatus => "/session",
            Self::SessionBudget => "/session budget",
            Self::SessionMemory => "/session memory",
            Self::SessionFeedback => "/session feedback",
            Self::JobStatus => "/job",
            Self::JobsSummary => "/jobs",
            Self::BackgroundSubmit => "/bg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManagedControlCommand {
    Reset,
    ResumeRestore,
    ResumeStatus,
    ResumeDrop,
    SessionAdmin,
    SessionInjection,
    SessionPartition,
}

impl ManagedControlCommand {
    pub(crate) const fn canonical_command(self) -> &'static str {
        match self {
            Self::Reset => "/reset",
            Self::ResumeRestore => "/resume",
            Self::ResumeStatus => "/resume status",
            Self::ResumeDrop => "/resume drop",
            Self::SessionAdmin => "/session admin",
            Self::SessionInjection => "/session inject",
            Self::SessionPartition => "/session partition",
        }
    }
}
