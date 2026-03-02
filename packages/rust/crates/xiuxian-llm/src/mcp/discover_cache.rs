//! Discover tool-call read-through cache backed by Valkey.
//!
//! This cache is intentionally scoped to discover-like calls (e.g. `skill.discover`)
//! to keep key cardinality predictable while accelerating repeated intent lookups.

use std::sync::Arc;

use anyhow::{Context, Result};
use redis::FromRedisValue;
use rmcp::model::{CallToolResult, Content, Meta};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, RwLock};

/// Runtime snapshot for discover cache backend.
#[derive(Debug, Clone)]
pub struct DiscoverCacheRuntimeInfo {
    /// Backend name.
    pub backend: &'static str,
    /// Cache TTL in seconds.
    pub ttl_secs: u64,
}

/// Discover cache backend configuration.
#[derive(Debug, Clone)]
pub struct DiscoverCacheConfig {
    /// Valkey connection URL.
    pub valkey_url: String,
    /// Cache key namespace prefix.
    pub key_prefix: String,
    /// Per-entry cache TTL in seconds.
    pub ttl_secs: u64,
}

/// Read-through discover cache facade.
#[derive(Debug)]
pub struct DiscoverReadThroughCache {
    backend: ValkeyDiscoverCache,
}

impl DiscoverReadThroughCache {
    /// Build discover cache from explicit config.
    ///
    /// # Errors
    /// Returns an error when valkey client initialization fails.
    pub fn from_config(config: DiscoverCacheConfig) -> Result<Self> {
        let backend = ValkeyDiscoverCache::new(config)?;
        Ok(Self { backend })
    }

    /// Return runtime details for diagnostics.
    #[must_use]
    pub fn runtime_info(&self) -> DiscoverCacheRuntimeInfo {
        DiscoverCacheRuntimeInfo {
            backend: "valkey",
            ttl_secs: self.backend.ttl_secs(),
        }
    }

    /// Build cache key for discover-like tool calls.
    #[must_use]
    pub fn build_cache_key(&self, tool_name: &str, arguments: Option<&Value>) -> Option<String> {
        if !is_discover_tool(tool_name) {
            return None;
        }

        let arguments = arguments?;
        let query = extract_discover_query(arguments)?;
        let normalized_args = canonicalize_json_value(arguments);
        let args_payload = normalized_args.to_string();
        let args_digest = sha256_hex(args_payload.as_bytes());
        let query_digest = sha256_hex(query.as_bytes());
        let tool_digest = tool_name.replace('.', "_");
        Some(format!(
            "{}:v1:{}:{}:{}",
            self.backend.key_prefix(),
            tool_digest,
            query_digest,
            args_digest
        ))
    }

    /// Read cached discover result.
    ///
    /// # Errors
    /// Returns an error when valkey access/decoding fails.
    pub async fn get(&self, key: &str) -> Result<Option<CallToolResult>> {
        self.backend.get(key).await
    }

    /// Write discover result into cache.
    ///
    /// # Errors
    /// Returns an error when valkey access/encoding fails.
    pub async fn set(&self, key: &str, result: &CallToolResult) -> Result<()> {
        self.backend.set(key, result).await
    }
}

#[derive(Debug)]
struct ValkeyDiscoverCache {
    client: redis::Client,
    key_prefix: String,
    ttl_secs: u64,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl ValkeyDiscoverCache {
    fn new(config: DiscoverCacheConfig) -> Result<Self> {
        let client = redis::Client::open(config.valkey_url.as_str()).with_context(|| {
            format!(
                "invalid valkey url for discover cache: {}",
                config.valkey_url
            )
        })?;
        Ok(Self {
            client,
            key_prefix: config.key_prefix,
            ttl_secs: config.ttl_secs,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        })
    }

    fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    fn ttl_secs(&self) -> u64 {
        self.ttl_secs
    }

    async fn acquire_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let _reconnect_guard = self.reconnect_lock.lock().await;
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("failed to open redis connection for discover cache")?;
        {
            let mut guard = self.connection.write().await;
            *guard = Some(connection.clone());
        }
        Ok(connection)
    }

    async fn invalidate_connection(&self) {
        let mut guard = self.connection.write().await;
        *guard = None;
    }

    async fn run_command<T, F>(&self, operation: &'static str, build: F) -> Result<T>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_error: Option<anyhow::Error> = None;
        for attempt in 0..2 {
            let mut conn = self.acquire_connection().await?;
            let result: redis::RedisResult<T> = build().query_async(&mut conn).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    tracing::warn!(
                        event = "mcp.pool.discover_cache.command.retry",
                        operation,
                        attempt = attempt + 1,
                        error = %error,
                        "discover cache valkey command failed; reconnecting"
                    );
                    self.invalidate_connection().await;
                    last_error = Some(
                        anyhow::anyhow!(error).context("discover cache valkey command failed"),
                    );
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("discover cache valkey command failed")))
    }

    async fn get(&self, key: &str) -> Result<Option<CallToolResult>> {
        let key = key.to_string();
        let raw: Option<String> = self
            .run_command("discover_cache_get", move || {
                let mut cmd = redis::cmd("GET");
                cmd.arg(&key);
                cmd
            })
            .await?;
        raw.map(|payload| {
            serde_json::from_str::<CachedCallToolResult>(&payload)
                .context("failed to decode cached discover tool result")
                .map(Into::into)
        })
        .transpose()
    }

    async fn set(&self, key: &str, result: &CallToolResult) -> Result<()> {
        let key = key.to_string();
        let ttl_secs = self.ttl_secs;
        let payload = serde_json::to_string(&CachedCallToolResult::from(result))
            .context("failed to encode discover cache payload")?;
        let _: () = self
            .run_command("discover_cache_set", move || {
                let mut cmd = redis::cmd("SETEX");
                cmd.arg(&key).arg(ttl_secs).arg(&payload);
                cmd
            })
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedCallToolResult {
    content: Vec<Content>,
    structured_content: Option<Value>,
    is_error: Option<bool>,
    meta: Option<Meta>,
}

impl From<&CallToolResult> for CachedCallToolResult {
    fn from(value: &CallToolResult) -> Self {
        Self {
            content: value.content.clone(),
            structured_content: value.structured_content.clone(),
            is_error: value.is_error,
            meta: value.meta.clone(),
        }
    }
}

impl From<CachedCallToolResult> for CallToolResult {
    fn from(value: CachedCallToolResult) -> Self {
        Self {
            content: value.content,
            structured_content: value.structured_content,
            is_error: value.is_error,
            meta: value.meta,
        }
    }
}

fn is_discover_tool(name: &str) -> bool {
    matches!(name.trim(), "skill.discover" | "skill_discover")
}

fn extract_discover_query(arguments: &Value) -> Option<String> {
    let object = arguments.as_object()?;
    let intent = object
        .get("intent")
        .and_then(Value::as_str)
        .or_else(|| object.get("query").and_then(Value::as_str))?;
    let trimmed = intent.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries = map.iter().collect::<Vec<_>>();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            let mut out = serde_json::Map::with_capacity(entries.len());
            for (key, child) in entries {
                out.insert(key.clone(), canonicalize_json_value(child));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_json_value).collect()),
        _ => value.clone(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
