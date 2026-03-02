//! Global swarm discovery via Valkey heartbeat registry.

use anyhow::{Context, Result, anyhow};
use rand::seq::SliceRandom;
use redis::FromRedisValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;

const REGISTRY_INDEX_KEY: &str = "xiuxian:swarm:registry:index";

/// Immutable identity published by one remote swarm worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterNodeIdentity {
    /// Cluster identifier (for example region or deployment id).
    pub cluster_id: String,
    /// Agent identifier unique within one cluster.
    pub agent_id: String,
    /// Routing role class (for example `teacher`, `steward`).
    pub role_class: String,
    /// Optional region hint for cross-DC routing.
    pub region: Option<String>,
    /// Optional endpoint hint for remote invocation.
    pub endpoint: Option<String>,
    /// Optional capability labels published by this worker.
    pub capabilities: Vec<String>,
}

impl ClusterNodeIdentity {
    fn sanitize(self) -> Self {
        Self {
            cluster_id: self.cluster_id.trim().to_string(),
            agent_id: self.agent_id.trim().to_string(),
            role_class: self.role_class.trim().to_ascii_lowercase(),
            region: normalize_optional_text(self.region),
            endpoint: normalize_optional_text(self.endpoint),
            capabilities: self
                .capabilities
                .into_iter()
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty())
                .collect(),
        }
    }
}

/// Resolved heartbeat record loaded from the global registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterNodeRecord {
    /// Registry key used by this record.
    pub registry_key: String,
    /// Published identity fields.
    pub identity: ClusterNodeIdentity,
    /// Last heartbeat timestamp in milliseconds.
    pub last_seen_ms: u64,
    /// Optional opaque metadata JSON published by the node.
    pub metadata: serde_json::Value,
}

/// Valkey-backed global discovery registry.
pub struct GlobalSwarmRegistry {
    redis_url: String,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl GlobalSwarmRegistry {
    /// Creates a discovery registry using the provided Valkey URL.
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    fn node_key(identity: &ClusterNodeIdentity) -> String {
        format!(
            "xiuxian:swarm:registry:{}:{}",
            identity.cluster_id, identity.agent_id
        )
    }

    fn heartbeat_payload(
        identity: &ClusterNodeIdentity,
        metadata: &serde_json::Value,
    ) -> Result<HashMap<String, String>> {
        if identity.cluster_id.trim().is_empty() {
            return Err(anyhow!("cluster_id must not be empty"));
        }
        if identity.agent_id.trim().is_empty() {
            return Err(anyhow!("agent_id must not be empty"));
        }
        if identity.role_class.trim().is_empty() {
            return Err(anyhow!("role_class must not be empty"));
        }

        let capabilities = serde_json::to_string(&identity.capabilities)?;
        let metadata_json = serde_json::to_string(metadata)?;
        let mut fields = HashMap::new();
        fields.insert("cluster_id".to_string(), identity.cluster_id.clone());
        fields.insert("agent_id".to_string(), identity.agent_id.clone());
        fields.insert("role_class".to_string(), identity.role_class.clone());
        fields.insert(
            "region".to_string(),
            identity.region.clone().unwrap_or_default(),
        );
        fields.insert(
            "endpoint".to_string(),
            identity.endpoint.clone().unwrap_or_default(),
        );
        fields.insert("capabilities".to_string(), capabilities);
        fields.insert("metadata".to_string(), metadata_json);
        fields.insert(
            "last_seen_ms".to_string(),
            current_unix_millis().to_string(),
        );
        Ok(fields)
    }

    /// Writes one heartbeat lease into the global registry.
    ///
    /// # Errors
    ///
    /// Returns an error when input fields are invalid or any Valkey command fails.
    pub async fn heartbeat(
        &self,
        identity: &ClusterNodeIdentity,
        metadata: &serde_json::Value,
        ttl_seconds: u64,
    ) -> Result<()> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        let sanitized = identity.clone().sanitize();
        let key = Self::node_key(&sanitized);
        let fields = Self::heartbeat_payload(&sanitized, metadata)?;

        for (field, value) in &fields {
            let _: i64 = self
                .run_command("swarm_registry_hset", || {
                    let mut command = redis::cmd("HSET");
                    command.arg(&key).arg(field).arg(value);
                    command
                })
                .await?;
        }

        let _: bool = self
            .run_command("swarm_registry_expire", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&key).arg(ttl_seconds);
                command
            })
            .await?;
        let _: i64 = self
            .run_command("swarm_registry_index_add", || {
                let mut command = redis::cmd("SADD");
                command.arg(REGISTRY_INDEX_KEY).arg(&key);
                command
            })
            .await?;
        Ok(())
    }

    /// Discovers all live nodes from the global registry.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub async fn discover_all(&self) -> Result<Vec<ClusterNodeRecord>> {
        self.discover(Some("*")).await
    }

    /// Discovers live nodes by role class.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub async fn discover_by_role(&self, role_class: &str) -> Result<Vec<ClusterNodeRecord>> {
        let normalized = role_class.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(Vec::new());
        }
        self.discover(Some(normalized.as_str())).await
    }

    /// Picks one live remote node matching a role class.
    ///
    /// Returns `None` when no candidate is available.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey access fails.
    pub async fn pick_candidate(
        &self,
        role_class: &str,
        exclude_cluster_id: Option<&str>,
    ) -> Result<Option<ClusterNodeRecord>> {
        let mut records = self.discover_by_role(role_class).await?;
        if let Some(exclude) = normalize_optional_text(exclude_cluster_id.map(ToString::to_string))
        {
            records.retain(|record| record.identity.cluster_id != exclude);
        }
        let mut rng = rand::thread_rng();
        Ok(records.choose(&mut rng).cloned())
    }

    /// Spawns a background heartbeat loop for one node identity.
    ///
    /// # Errors
    ///
    /// Returns an error when `ttl_seconds` or `interval` is invalid.
    pub fn spawn_heartbeat_loop(
        self: Arc<Self>,
        identity: ClusterNodeIdentity,
        metadata: serde_json::Value,
        ttl_seconds: u64,
        interval: Duration,
    ) -> Result<tokio::task::JoinHandle<()>> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        if interval.is_zero() {
            return Err(anyhow!("heartbeat interval must be > 0"));
        }
        let min_ttl = interval.as_secs().saturating_mul(2);
        if ttl_seconds <= min_ttl {
            return Err(anyhow!(
                "ttl_seconds must be at least 2x interval (ttl={ttl_seconds}, interval_secs={})",
                interval.as_secs()
            ));
        }

        let handle = tokio::spawn(async move {
            loop {
                if let Err(error) = self.heartbeat(&identity, &metadata, ttl_seconds).await {
                    log::warn!("swarm heartbeat failed for {}: {error}", identity.agent_id);
                }
                tokio::time::sleep(interval).await;
            }
        });
        Ok(handle)
    }

    async fn discover(&self, role_filter: Option<&str>) -> Result<Vec<ClusterNodeRecord>> {
        let keys: Vec<String> = self
            .run_command("swarm_registry_index_members", || {
                let mut command = redis::cmd("SMEMBERS");
                command.arg(REGISTRY_INDEX_KEY);
                command
            })
            .await?;
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();
        let mut stale = Vec::new();
        for key in keys {
            let fields: HashMap<String, String> = self
                .run_command("swarm_registry_hgetall", || {
                    let mut command = redis::cmd("HGETALL");
                    command.arg(&key);
                    command
                })
                .await?;
            if fields.is_empty() {
                stale.push(key);
                continue;
            }

            if let Some(record) = parse_record(key, &fields)
                && role_matches(role_filter, &record.identity.role_class)
            {
                records.push(record);
            }
        }

        if !stale.is_empty() {
            for key in stale {
                let _: i64 = self
                    .run_command("swarm_registry_prune_stale_index", || {
                        let mut command = redis::cmd("SREM");
                        command.arg(REGISTRY_INDEX_KEY).arg(&key);
                        command
                    })
                    .await?;
            }
        }

        Ok(records)
    }

    async fn run_command<T, F>(&self, operation: &'static str, build: F) -> Result<T>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_error: Option<redis::RedisError> = None;
        for _ in 0..2 {
            let mut connection = self.acquire_connection().await?;
            let command = build();
            let result: redis::RedisResult<T> = command.query_async(&mut connection).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    self.invalidate_connection().await;
                    last_error = Some(error);
                }
            }
        }
        match last_error {
            Some(error) => Err(anyhow!("valkey {operation} failed: {error}")),
            None => Err(anyhow!("valkey {operation} failed unexpectedly")),
        }
    }

    async fn acquire_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let _guard = self.reconnect_lock.lock().await;
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let client = redis::Client::open(self.redis_url.as_str())
            .context("Failed to connect to Valkey for swarm discovery")?;
        let connection = client.get_multiplexed_async_connection().await?;
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
}

fn role_matches(role_filter: Option<&str>, role_class: &str) -> bool {
    match role_filter {
        Some("*") | None => true,
        Some(value) => value.eq_ignore_ascii_case(role_class),
    }
}

fn parse_record(
    registry_key: String,
    fields: &HashMap<String, String>,
) -> Option<ClusterNodeRecord> {
    let cluster_id = fields.get("cluster_id")?.trim().to_string();
    let agent_id = fields.get("agent_id")?.trim().to_string();
    let role_class = fields.get("role_class")?.trim().to_ascii_lowercase();
    if cluster_id.is_empty() || agent_id.is_empty() || role_class.is_empty() {
        return None;
    }

    let capabilities = fields
        .get("capabilities")
        .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
        .unwrap_or_default();
    let metadata = fields
        .get("metadata")
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .unwrap_or(serde_json::Value::Null);
    let last_seen_ms = fields
        .get("last_seen_ms")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or_default();

    Some(ClusterNodeRecord {
        registry_key,
        identity: ClusterNodeIdentity {
            cluster_id,
            agent_id,
            role_class,
            region: normalize_optional_text(fields.get("region").cloned()),
            endpoint: normalize_optional_text(fields.get("endpoint").cloned()),
            capabilities,
        },
        last_seen_ms,
        metadata,
    })
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|inner| inner.trim().to_string())
        .and_then(|inner| if inner.is_empty() { None } else { Some(inner) })
}

fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
