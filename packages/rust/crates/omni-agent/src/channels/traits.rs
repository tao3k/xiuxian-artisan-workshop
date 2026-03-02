//! Channel trait and message types for chat platforms (Telegram, Discord, etc.).

use async_trait::async_trait;

/// Mutation operation for recipient-scoped delegated command-admin identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipientCommandAdminUsersMutation {
    /// Replace override list with the provided identities.
    Set(Vec<String>),
    /// Add identities to current override list.
    Add(Vec<String>),
    /// Remove identities from current override list.
    Remove(Vec<String>),
    /// Clear recipient-scoped override list.
    Clear,
}

/// A message received from or sent to a channel.
#[derive(Debug, Clone)]
pub struct ChannelMessage {
    /// Unique message ID (e.g. `telegram_{chat_id}_{message_id}` to prevent duplicate memories).
    pub id: String,
    /// Sender identifier (username or `user_id` string) for logs and diagnostics.
    pub sender: String,
    /// Reply target for channel send operations (for Telegram, this is `chat_id`).
    pub recipient: String,
    /// Session partition key (Telegram default: `chat_id:user_id`; configurable by channel strategy).
    pub session_key: String,
    /// Message text content.
    pub content: String,
    /// Channel name (e.g. `telegram`).
    pub channel: String,
    /// Unix timestamp.
    pub timestamp: u64,
}

/// Core channel trait — implement for any messaging platform.
#[async_trait]
pub trait Channel: Send + Sync {
    /// Human-readable channel name.
    fn name(&self) -> &str;

    /// Optional session partition mode label for diagnostics (`chat_user`, `chat`, ...).
    fn session_partition_mode(&self) -> Option<String> {
        None
    }

    /// Optional runtime session partition mode update (`chat`, `chat_user`, ...).
    ///
    /// # Errors
    /// Returns an error when the channel implementation does not support runtime partition updates.
    fn set_session_partition_mode(&self, _mode: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "runtime session partition update is not supported for this channel"
        ))
    }

    /// Whether this sender identity is allowed to run privileged control commands.
    fn is_admin_user(&self, _identity: &str) -> bool {
        false
    }

    /// Whether this sender identity is allowed to run a specific privileged control command.
    ///
    /// Default behavior delegates to `is_admin_user`, while channel implementations can
    /// apply per-command authorization rules.
    fn is_authorized_for_control_command(&self, identity: &str, _command_text: &str) -> bool {
        self.is_admin_user(identity)
    }

    /// Recipient-aware variant of control-command authorization.
    ///
    /// Default behavior keeps backward compatibility by delegating to
    /// `is_authorized_for_control_command`.
    fn is_authorized_for_control_command_for_recipient(
        &self,
        identity: &str,
        command_text: &str,
        _recipient: &str,
    ) -> bool {
        self.is_authorized_for_control_command(identity, command_text)
    }

    /// Whether this sender identity is allowed to run a specific non-privileged slash command.
    ///
    /// Default behavior allows all identities. Channel implementations can override this
    /// for command-scoped ACL policies.
    fn is_authorized_for_slash_command(&self, _identity: &str, _command_scope: &str) -> bool {
        true
    }

    /// Recipient-aware variant of slash-command authorization.
    ///
    /// Default behavior keeps backward compatibility by delegating to
    /// `is_authorized_for_slash_command`.
    fn is_authorized_for_slash_command_for_recipient(
        &self,
        identity: &str,
        command_scope: &str,
        _recipient: &str,
    ) -> bool {
        self.is_authorized_for_slash_command(identity, command_scope)
    }

    /// Returns recipient-scoped delegated command admins override.
    ///
    /// `Ok(None)` means no recipient override (fallback to global ACL chain).
    ///
    /// # Errors
    /// Returns an error when recipient-scoped override lookup is unsupported.
    fn recipient_command_admin_users(
        &self,
        _recipient: &str,
    ) -> anyhow::Result<Option<Vec<String>>> {
        Err(anyhow::anyhow!(
            "recipient-scoped command admin overrides are not supported for this channel"
        ))
    }

    /// Mutates recipient-scoped delegated command admins override.
    ///
    /// Returns the updated override list; `None` means override cleared.
    ///
    /// # Errors
    /// Returns an error when recipient-scoped override mutation is unsupported.
    fn mutate_recipient_command_admin_users(
        &self,
        _recipient: &str,
        _mutation: RecipientCommandAdminUsersMutation,
    ) -> anyhow::Result<Option<Vec<String>>> {
        Err(anyhow::anyhow!(
            "recipient-scoped command admin overrides are not supported for this channel"
        ))
    }

    /// Send a message through this channel.
    ///
    /// # Errors
    /// Returns an error when channel send fails.
    async fn send(&self, message: &str, recipient: &str) -> anyhow::Result<()>;

    /// Start listening for incoming messages (long-running).
    ///
    /// # Errors
    /// Returns an error when channel listen loop setup or polling fails.
    async fn listen(&self, tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> anyhow::Result<()>;

    /// Check if channel is healthy.
    ///
    /// Returns `true` when channel transport is currently healthy.
    async fn health_check(&self) -> bool {
        true
    }

    /// Signal that the bot is processing a response (e.g. "typing" indicator).
    ///
    /// # Errors
    /// Returns an error when typing-indicator transport fails.
    async fn start_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Stop any active typing indicator.
    ///
    /// # Errors
    /// Returns an error when typing-indicator stop transport fails.
    async fn stop_typing(&self, _recipient: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
