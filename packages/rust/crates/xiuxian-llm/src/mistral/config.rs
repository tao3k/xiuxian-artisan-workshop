//! Mistral server runtime configuration.

const DEFAULT_COMMAND: &str = "mistralrs-server";
const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11500";
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 45;
const DEFAULT_PROBE_TIMEOUT_MS: u64 = 1_500;
const DEFAULT_PROBE_INTERVAL_MS: u64 = 250;

/// Runtime settings for managing a `mistralrs-server` process.
#[derive(Debug, Clone)]
pub struct MistralServerConfig {
    /// Executable path or command name (for example `mistralrs-server`).
    pub command: String,
    /// CLI args passed to the server process.
    pub args: Vec<String>,
    /// OpenAI-compatible base URL exposed by the server.
    pub base_url: String,
    /// Max seconds to wait for readiness.
    pub startup_timeout_secs: u64,
    /// Timeout per health probe request.
    pub probe_timeout_ms: u64,
    /// Sleep interval between readiness probes.
    pub probe_interval_ms: u64,
}

impl Default for MistralServerConfig {
    fn default() -> Self {
        Self {
            command: DEFAULT_COMMAND.to_string(),
            args: Vec::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            startup_timeout_secs: DEFAULT_STARTUP_TIMEOUT_SECS,
            probe_timeout_ms: DEFAULT_PROBE_TIMEOUT_MS,
            probe_interval_ms: DEFAULT_PROBE_INTERVAL_MS,
        }
    }
}

impl MistralServerConfig {
    /// Build config from environment overrides.
    ///
    /// Supported variables:
    /// - `XIUXIAN_MISTRAL_SERVER_COMMAND`
    /// - `XIUXIAN_MISTRAL_SERVER_ARGS` (whitespace-separated)
    /// - `XIUXIAN_MISTRAL_SERVER_BASE_URL`
    /// - `XIUXIAN_MISTRAL_SERVER_STARTUP_TIMEOUT_SECS`
    /// - `XIUXIAN_MISTRAL_SERVER_PROBE_TIMEOUT_MS`
    /// - `XIUXIAN_MISTRAL_SERVER_PROBE_INTERVAL_MS`
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();
        if let Some(command) = non_empty_env("XIUXIAN_MISTRAL_SERVER_COMMAND") {
            config.command = command;
        }
        if let Some(raw_args) = non_empty_env("XIUXIAN_MISTRAL_SERVER_ARGS") {
            config.args = split_shell_like_args(&raw_args);
        }
        if let Some(base_url) = non_empty_env("XIUXIAN_MISTRAL_SERVER_BASE_URL") {
            config.base_url = base_url;
        }
        if let Some(timeout_secs) = parse_env_u64("XIUXIAN_MISTRAL_SERVER_STARTUP_TIMEOUT_SECS") {
            config.startup_timeout_secs = timeout_secs.max(1);
        }
        if let Some(timeout_ms) = parse_env_u64("XIUXIAN_MISTRAL_SERVER_PROBE_TIMEOUT_MS") {
            config.probe_timeout_ms = timeout_ms.max(1);
        }
        if let Some(interval_ms) = parse_env_u64("XIUXIAN_MISTRAL_SERVER_PROBE_INTERVAL_MS") {
            config.probe_interval_ms = interval_ms.max(1);
        }
        config
    }
}

fn parse_env_u64(name: &str) -> Option<u64> {
    non_empty_env(name).and_then(|raw| raw.parse::<u64>().ok())
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

fn split_shell_like_args(raw: &str) -> Vec<String> {
    raw.split_whitespace().map(ToString::to_string).collect()
}
