use crate::config::load_runtime_settings;

pub(super) const DEFAULT_SESSION_KEY_PREFIX: &str = "omni-agent:session";

#[derive(Debug, Clone)]
pub(crate) struct RedisSessionConfig {
    pub(crate) url: String,
    pub(crate) key_prefix: String,
    pub(crate) ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RedisSessionRuntimeSnapshot {
    pub(crate) url: String,
    pub(crate) key_prefix: String,
    pub(crate) ttl_secs: Option<u64>,
}

impl RedisSessionConfig {
    pub(crate) fn from_env() -> Option<Self> {
        let settings = load_runtime_settings();
        let url = settings
            .session
            .valkey_url
            .as_deref()
            .map(str::trim)
            .map(str::to_string)
            .filter(|v| !v.is_empty())
            .or_else(|| {
                std::env::var("VALKEY_URL")
                    .ok()
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty())
            })?;
        let key_prefix = std::env::var("OMNI_AGENT_SESSION_VALKEY_PREFIX")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .or_else(|| {
                settings
                    .session
                    .redis_prefix
                    .as_deref()
                    .map(str::trim)
                    .map(str::to_string)
                    .filter(|v| !v.is_empty())
            })
            .unwrap_or_else(|| DEFAULT_SESSION_KEY_PREFIX.to_string());
        let ttl_secs = match std::env::var("OMNI_AGENT_SESSION_TTL_SECS") {
            Ok(raw) => match raw.parse::<u64>() {
                Ok(v) if v > 0 => Some(v),
                _ => {
                    tracing::warn!(
                        env_var = "OMNI_AGENT_SESSION_TTL_SECS",
                        value = %raw,
                        "invalid session ttl env value; using settings/default"
                    );
                    settings.session.ttl_secs.filter(|v| *v > 0)
                }
            },
            Err(_) => settings.session.ttl_secs.filter(|v| *v > 0),
        };
        Some(Self {
            url,
            key_prefix,
            ttl_secs,
        })
    }
}
