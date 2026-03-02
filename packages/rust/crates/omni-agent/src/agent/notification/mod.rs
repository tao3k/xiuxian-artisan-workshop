//! Pluggable notification dispatch for agent-side reminders.

mod dispatcher;
mod provider;
mod recipient;

pub use dispatcher::NotificationDispatcher;
pub use provider::NotificationProvider;

pub(crate) use recipient::{
    parse_prefixed_recipient, recipient_is_telegram_chat_id, recipient_target_for,
};

pub mod discord;
pub mod linux;
pub mod llm;
pub mod telegram;
