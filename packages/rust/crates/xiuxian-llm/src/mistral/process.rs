//! Process lifecycle management for `mistralrs-server`.

use std::process::{Child, Command, Stdio};
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use tokio::time::{Duration, Instant as TokioInstant, sleep};

use super::config::MistralServerConfig;
use super::health::probe_models;

/// Managed `mistralrs-server` child process with readiness gating.
pub struct ManagedMistralServer {
    config: MistralServerConfig,
    child: Child,
    started_at: Instant,
}

/// Spawn `mistralrs-server` and wait until `/v1/models` is ready.
///
/// # Errors
/// Returns an error when process spawn fails, process exits before ready,
/// or readiness timeout is exceeded.
pub async fn spawn_mistral_server(config: MistralServerConfig) -> Result<ManagedMistralServer> {
    ManagedMistralServer::spawn(config).await
}

impl ManagedMistralServer {
    /// Spawn `mistralrs-server` and wait until `/v1/models` is ready.
    ///
    /// # Errors
    /// Returns an error when process spawn fails, process exits before ready,
    /// or readiness timeout is exceeded.
    pub async fn spawn(config: MistralServerConfig) -> Result<Self> {
        let command_display = format!("{} {}", config.command, config.args.join(" "));
        let child = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| {
                format!("failed to spawn mistral server command: {command_display}")
            })?;

        let mut managed = Self {
            config,
            child,
            started_at: Instant::now(),
        };
        managed.wait_until_ready().await?;
        Ok(managed)
    }

    /// Wait until OpenAI-compatible endpoint is ready.
    ///
    /// # Errors
    /// Returns an error if process exits early or readiness timeout is exceeded.
    pub async fn wait_until_ready(&mut self) -> Result<()> {
        let deadline =
            TokioInstant::now() + Duration::from_secs(self.config.startup_timeout_secs.max(1));

        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .context("failed to query mistral server child status")?
            {
                return Err(anyhow!("mistral server exited before ready: {status}"));
            }

            let probe = probe_models(&self.config.base_url, self.config.probe_timeout_ms).await;
            if probe.ready {
                tracing::info!(
                    event = "xiuxian.llm.mistral.ready",
                    pid = self.child.id(),
                    base_url = %self.config.base_url,
                    startup_elapsed_ms = self.started_at.elapsed().as_millis(),
                    summary = %probe.summary,
                    "mistral server ready"
                );
                return Ok(());
            }

            if TokioInstant::now() >= deadline {
                return Err(anyhow!(
                    "mistral server readiness timed out after {}s ({})",
                    self.config.startup_timeout_secs.max(1),
                    probe.summary
                ));
            }

            tracing::debug!(
                event = "xiuxian.llm.mistral.wait_ready",
                pid = self.child.id(),
                base_url = %self.config.base_url,
                summary = %probe.summary,
                "mistral server not ready yet"
            );
            sleep(Duration::from_millis(self.config.probe_interval_ms.max(1))).await;
        }
    }

    /// Return server process id.
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Return configured OpenAI-compatible base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Stop child process (best effort) and wait for exit.
    ///
    /// # Errors
    /// Returns an error when process control operations fail.
    pub fn stop(&mut self) -> Result<()> {
        if self
            .child
            .try_wait()
            .context("failed to query mistral server child status")?
            .is_some()
        {
            return Ok(());
        }
        self.child
            .kill()
            .context("failed to kill mistral server process")?;
        let _ = self.child.wait();
        Ok(())
    }
}

impl Drop for ManagedMistralServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
