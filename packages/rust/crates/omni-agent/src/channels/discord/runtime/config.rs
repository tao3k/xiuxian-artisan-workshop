use super::super::session_partition::DiscordSessionPartition;
use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::config::{DiscordSettings, load_runtime_settings};
use xiuxian_macros::env_non_empty;

const DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY: usize = 512;
const DISCORD_DEFAULT_TURN_TIMEOUT_SECS: u64 = 120;
const DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES: usize = 16;

/// Runtime configuration for Discord ingress/dispatch loop.
#[derive(Debug, Clone)]
pub struct DiscordRuntimeConfig {
    /// Session partition strategy used for Discord messages.
    pub session_partition: DiscordSessionPartition,
    /// Inbound ingress queue capacity.
    pub inbound_queue_capacity: usize,
    /// Per-turn timeout in seconds.
    pub turn_timeout_secs: u64,
    /// Maximum number of in-flight foreground messages.
    pub foreground_max_in_flight_messages: usize,
    /// Foreground queue mode for same-session inbound messages.
    pub foreground_queue_mode: ForegroundQueueMode,
}

impl DiscordRuntimeConfig {
    /// Resolve Discord runtime config from environment-backed defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let settings = load_runtime_settings();
        Self::from_lookup(|name| env_non_empty!(name), Some(&settings.discord))
    }

    fn from_lookup<F>(lookup: F, settings: Option<&DiscordSettings>) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let defaults = Self::default();
        Self {
            session_partition: DiscordSessionPartition::from_env(),
            inbound_queue_capacity: resolve_usize(
                &lookup,
                "OMNI_AGENT_DISCORD_INBOUND_QUEUE_CAPACITY",
                settings.and_then(|s| s.inbound_queue_capacity),
                defaults.inbound_queue_capacity,
            ),
            turn_timeout_secs: resolve_u64(
                &lookup,
                "OMNI_AGENT_DISCORD_TURN_TIMEOUT_SECS",
                settings.and_then(|s| s.turn_timeout_secs),
                defaults.turn_timeout_secs,
            ),
            foreground_max_in_flight_messages: resolve_usize(
                &lookup,
                "OMNI_AGENT_DISCORD_FOREGROUND_MAX_IN_FLIGHT_MESSAGES",
                settings.and_then(|s| s.foreground_max_in_flight_messages),
                defaults.foreground_max_in_flight_messages,
            ),
            foreground_queue_mode: resolve_foreground_queue_mode(
                &lookup,
                "OMNI_AGENT_DISCORD_FOREGROUND_QUEUE_MODE",
                settings.and_then(|s| s.foreground_queue_mode.as_deref()),
                defaults.foreground_queue_mode,
            ),
        }
    }
}

impl Default for DiscordRuntimeConfig {
    fn default() -> Self {
        Self {
            session_partition: DiscordSessionPartition::from_env(),
            inbound_queue_capacity: DISCORD_DEFAULT_INBOUND_QUEUE_CAPACITY,
            turn_timeout_secs: DISCORD_DEFAULT_TURN_TIMEOUT_SECS,
            foreground_max_in_flight_messages: DISCORD_DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES,
            foreground_queue_mode: ForegroundQueueMode::Interrupt,
        }
    }
}

fn resolve_usize<F>(lookup: &F, name: &str, setting_value: Option<usize>, default: usize) -> usize
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup(name) {
        match raw.trim().parse::<usize>() {
            Ok(value) if value > 0 => return value,
            _ => tracing::warn!(
                env_var = %name,
                value = %raw,
                "invalid runtime config env value; using settings/default"
            ),
        }
    }
    match setting_value {
        Some(value) if value > 0 => value,
        Some(value) => {
            tracing::warn!(
                setting = %name,
                value,
                default,
                "invalid runtime config settings value; using default"
            );
            default
        }
        None => default,
    }
}

fn resolve_u64<F>(lookup: &F, name: &str, setting_value: Option<u64>, default: u64) -> u64
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup(name) {
        match raw.trim().parse::<u64>() {
            Ok(value) if value > 0 => return value,
            _ => tracing::warn!(
                env_var = %name,
                value = %raw,
                "invalid runtime config env value; using settings/default"
            ),
        }
    }
    match setting_value {
        Some(value) if value > 0 => value,
        Some(value) => {
            tracing::warn!(
                setting = %name,
                value,
                default,
                "invalid runtime config settings value; using default"
            );
            default
        }
        None => default,
    }
}

fn resolve_foreground_queue_mode<F>(
    lookup: &F,
    env_name: &str,
    setting_value: Option<&str>,
    default: ForegroundQueueMode,
) -> ForegroundQueueMode
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(raw) = lookup(env_name) {
        if let Some(mode) = ForegroundQueueMode::parse(raw.as_str()) {
            return mode;
        }
        tracing::warn!(
            env_var = %env_name,
            value = %raw,
            "invalid queue mode env value; using settings/default"
        );
    }
    if let Some(raw) = setting_value {
        if let Some(mode) = ForegroundQueueMode::parse(raw) {
            return mode;
        }
        tracing::warn!(
            setting = %env_name,
            value = %raw,
            "invalid queue mode settings value; using default"
        );
    }
    default
}
