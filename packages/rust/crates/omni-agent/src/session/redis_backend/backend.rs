use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use tokio::sync::{Mutex, RwLock};

use super::config::{DEFAULT_SESSION_KEY_PREFIX, RedisSessionConfig, RedisSessionRuntimeSnapshot};

pub(super) fn usize_to_i64_saturating(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

pub(super) fn usize_to_u64_saturating(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

#[derive(Debug)]
pub(crate) struct RedisSessionBackend {
    pub(super) client: redis::Client,
    pub(super) url: String,
    pub(super) key_prefix: String,
    pub(super) ttl_secs: Option<u64>,
    pub(super) message_content_max_chars: Option<usize>,
    pub(super) connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    pub(super) reconnect_lock: Arc<Mutex<()>>,
}

impl RedisSessionBackend {
    pub(crate) fn from_env() -> Option<Result<Self>> {
        let cfg = RedisSessionConfig::from_env()?;
        Some(Self::new(cfg))
    }

    pub(crate) fn new(cfg: RedisSessionConfig) -> Result<Self> {
        let client = redis::Client::open(cfg.url.as_str())
            .with_context(|| format!("invalid redis url for session backend: {}", cfg.url))?;
        Ok(Self {
            client,
            url: cfg.url,
            key_prefix: cfg.key_prefix,
            ttl_secs: cfg.ttl_secs,
            message_content_max_chars: cfg.message_content_max_chars,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        })
    }

    pub(crate) fn new_from_parts(
        url: String,
        key_prefix: Option<String>,
        ttl_secs: Option<u64>,
    ) -> Result<Self> {
        let prefix = key_prefix
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| DEFAULT_SESSION_KEY_PREFIX.to_string());
        Self::new(RedisSessionConfig {
            url,
            key_prefix: prefix,
            ttl_secs: ttl_secs.filter(|value| *value > 0),
            message_content_max_chars: None,
        })
    }

    pub(crate) fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    pub(crate) fn ttl_secs(&self) -> Option<u64> {
        self.ttl_secs
    }

    pub(crate) fn runtime_snapshot(&self) -> RedisSessionRuntimeSnapshot {
        RedisSessionRuntimeSnapshot {
            url: self.url.clone(),
            key_prefix: self.key_prefix.clone(),
            ttl_secs: self.ttl_secs,
            message_content_max_chars: self.message_content_max_chars,
        }
    }

    pub(super) fn messages_key(&self, session_id: &str) -> String {
        format!("{}:messages:{}", self.key_prefix, session_id)
    }

    pub(super) fn window_key(&self, session_id: &str) -> String {
        format!("{}:window:{}", self.key_prefix, session_id)
    }

    pub(super) fn summary_key(&self, session_id: &str) -> String {
        format!("{}:summary:{}", self.key_prefix, session_id)
    }

    pub(super) fn stream_key(&self, stream_name: &str) -> String {
        format!("{}:stream:{}", self.key_prefix, stream_name)
    }

    pub(super) fn stream_metrics_global_key(&self, stream_name: &str) -> String {
        format!("{}:metrics:{}", self.key_prefix, stream_name)
    }

    pub(super) fn stream_metrics_session_key(&self, stream_name: &str, session_id: &str) -> String {
        format!(
            "{}:metrics:{}:session:{}",
            self.key_prefix, stream_name, session_id
        )
    }

    pub(super) fn now_unix_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .unwrap_or(0)
    }
}
