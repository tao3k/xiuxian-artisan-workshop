use anyhow::{Context, Result};
use async_trait::async_trait;

use super::{NotificationProvider, recipient_target_for};

/// Local desktop notification provider via `notify-send`.
pub struct LinuxProvider;

impl LinuxProvider {
    /// Create a linux provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NotificationProvider for LinuxProvider {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn supports(&self, recipient: &str) -> bool {
        recipient_target_for(recipient, &["linux", "local", "os"]).is_some()
    }

    async fn send(&self, _recipient: &str, content: &str) -> Result<()> {
        let content = content.to_string();
        tokio::task::spawn_blocking(move || {
            let status = std::process::Command::new("notify-send")
                .arg("Xiuxian Reminder")
                .arg(content)
                .status()
                .context("failed to execute notify-send")?;
            if !status.success() {
                anyhow::bail!("notify-send exited with status {status}");
            }
            Ok(())
        })
        .await
        .context("notify-send task join failed")??;
        Ok(())
    }
}
