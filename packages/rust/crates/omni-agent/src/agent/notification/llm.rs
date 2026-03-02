use anyhow::Result;
use async_trait::async_trait;

use super::{NotificationProvider, parse_prefixed_recipient, recipient_target_for};

/// LLM-context fallback notification provider.
///
/// This provider is intentionally transport-agnostic and logs to runtime tracing.
pub struct LlmProvider;

impl LlmProvider {
    /// Create an llm-context provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NotificationProvider for LlmProvider {
    fn name(&self) -> &'static str {
        "llm"
    }

    fn supports(&self, recipient: &str) -> bool {
        recipient_target_for(recipient, &["llm"]).is_some()
            || parse_prefixed_recipient(recipient).is_none()
    }

    async fn send(&self, recipient: &str, content: &str) -> Result<()> {
        tracing::info!(
            event = "notification.dispatch.llm_context",
            recipient,
            content,
            "notification delivered to llm-context fallback sink"
        );
        Ok(())
    }
}
