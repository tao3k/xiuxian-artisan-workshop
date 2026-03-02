//! Queueing policy for foreground messages in managed chat runtimes.

use std::fmt;

/// Foreground queue mode applied when a new user message arrives for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ForegroundQueueMode {
    /// Keep the current foreground turn running and queue the new message.
    Queue,
    /// Interrupt the active foreground turn in the same session before queuing.
    #[default]
    Interrupt,
}

impl ForegroundQueueMode {
    /// Parse a queue mode from a settings/env string.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "queue" => Some(Self::Queue),
            "interrupt" => Some(Self::Interrupt),
            _ => None,
        }
    }

    /// Return the canonical string used in logs and runtime banners.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queue => "queue",
            Self::Interrupt => "interrupt",
        }
    }

    /// Whether the runtime should preempt an active turn on new inbound input.
    #[must_use]
    pub const fn should_interrupt_on_new_message(self) -> bool {
        matches!(self, Self::Interrupt)
    }
}

impl fmt::Display for ForegroundQueueMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
