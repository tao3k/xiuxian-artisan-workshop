//! Telegram foreground runtime configuration (queueing, concurrency, timeout).

use xiuxian_macros::env_non_empty;

use crate::channels::managed_runtime::ForegroundQueueMode;
use crate::config::{TelegramSettings, load_runtime_settings};

const DEFAULT_INBOUND_QUEUE_CAPACITY: usize = 100;
const DEFAULT_FOREGROUND_QUEUE_CAPACITY: usize = 256;
const DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES: usize = 16;
const DEFAULT_FOREGROUND_TURN_TIMEOUT_SECS: u64 = 80;

/// Effective Telegram foreground runtime limits after env/settings resolution.
#[derive(Debug, Clone, Copy)]
pub struct TelegramRuntimeConfig {
    /// Inbound webhook/polling queue capacity.
    pub inbound_queue_capacity: usize,
    /// Foreground processing queue capacity.
    pub foreground_queue_capacity: usize,
    /// Maximum number of in-flight foreground messages.
    pub foreground_max_in_flight_messages: usize,
    /// Foreground turn timeout in seconds.
    pub foreground_turn_timeout_secs: u64,
    /// Foreground queue mode for same-session inbound messages.
    pub foreground_queue_mode: ForegroundQueueMode,
}

impl Default for TelegramRuntimeConfig {
    fn default() -> Self {
        Self {
            inbound_queue_capacity: DEFAULT_INBOUND_QUEUE_CAPACITY,
            foreground_queue_capacity: DEFAULT_FOREGROUND_QUEUE_CAPACITY,
            foreground_max_in_flight_messages: DEFAULT_FOREGROUND_MAX_IN_FLIGHT_MESSAGES,
            foreground_turn_timeout_secs: DEFAULT_FOREGROUND_TURN_TIMEOUT_SECS,
            foreground_queue_mode: ForegroundQueueMode::Interrupt,
        }
    }
}

impl TelegramRuntimeConfig {
    /// Resolve runtime config from environment variables and settings defaults.
    #[must_use]
    pub fn from_env() -> Self {
        let settings = load_runtime_settings();
        Self::from_lookup(|name| env_non_empty!(name), Some(&settings.telegram))
    }

    #[doc(hidden)]
    pub fn from_lookup_for_test<F>(lookup: F, settings: Option<&TelegramSettings>) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        Self::from_lookup(lookup, settings)
    }

    fn from_lookup<F>(lookup: F, settings: Option<&TelegramSettings>) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let defaults = Self::default();
        Self {
            inbound_queue_capacity: resolve_usize(
                &lookup,
                "OMNI_AGENT_TELEGRAM_INBOUND_QUEUE_CAPACITY",
                settings.and_then(|s| s.inbound_queue_capacity),
                defaults.inbound_queue_capacity,
            ),
            foreground_queue_capacity: resolve_usize(
                &lookup,
                "OMNI_AGENT_TELEGRAM_FOREGROUND_QUEUE_CAPACITY",
                settings.and_then(|s| s.foreground_queue_capacity),
                defaults.foreground_queue_capacity,
            ),
            foreground_max_in_flight_messages: resolve_usize(
                &lookup,
                "OMNI_AGENT_TELEGRAM_FOREGROUND_MAX_IN_FLIGHT",
                settings.and_then(|s| s.foreground_max_in_flight_messages),
                defaults.foreground_max_in_flight_messages,
            ),
            foreground_turn_timeout_secs: resolve_u64(
                &lookup,
                "OMNI_AGENT_TELEGRAM_FOREGROUND_TURN_TIMEOUT_SECS",
                settings.and_then(|s| s.foreground_turn_timeout_secs),
                defaults.foreground_turn_timeout_secs,
            ),
            foreground_queue_mode: resolve_foreground_queue_mode(
                &lookup,
                "OMNI_AGENT_TELEGRAM_FOREGROUND_QUEUE_MODE",
                settings.and_then(|s| s.foreground_queue_mode.as_deref()),
                defaults.foreground_queue_mode,
            ),
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
