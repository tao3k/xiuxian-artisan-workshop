//! Cross-cluster remote possession protocol over Valkey.

use crate::contracts::{FlowInstruction, QianjiOutput};
use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use redis::FromRedisValue;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, timeout};

/// One remote node-execution request published by a source cluster.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteNodeRequest {
    /// Unique request id.
    pub request_id: String,
    /// Shared workflow session id.
    pub session_id: String,
    /// Target node id to execute remotely.
    pub node_id: String,
    /// Target role class that can execute this node.
    pub role_class: String,
    /// Requester cluster identifier.
    pub requester_cluster_id: String,
    /// Requester agent identifier.
    pub requester_agent_id: String,
    /// Serialized context snapshot at delegation point.
    pub context: serde_json::Value,
    /// Request timestamp in unix milliseconds.
    pub created_ms: u64,
}

impl RemoteNodeRequest {
    /// Creates a request with generated id and timestamp.
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        node_id: impl Into<String>,
        role_class: impl Into<String>,
        requester_cluster_id: impl Into<String>,
        requester_agent_id: impl Into<String>,
        context: serde_json::Value,
    ) -> Self {
        let created_ms = current_unix_millis();
        let random: u64 = rand::random();
        let request_id = format!("remote_possession_{created_ms}_{random:x}");
        Self {
            request_id,
            session_id: session_id.into(),
            node_id: node_id.into(),
            role_class: role_class.into(),
            requester_cluster_id: requester_cluster_id.into(),
            requester_agent_id: requester_agent_id.into(),
            context,
            created_ms,
        }
    }
}

/// One remote execution response returned by a responder cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteNodeResponse {
    /// Request id to correlate with caller.
    pub request_id: String,
    /// Session id carried from request.
    pub session_id: String,
    /// Target node id that was executed.
    pub node_id: String,
    /// Responder cluster id.
    pub responder_cluster_id: String,
    /// Responder agent id.
    pub responder_agent_id: String,
    /// Whether execution was successful.
    pub ok: bool,
    /// Output payload when successful.
    pub output: Option<QianjiOutput>,
    /// Error message when failed.
    pub error: Option<String>,
    /// Response timestamp in unix milliseconds.
    pub finished_ms: u64,
}

impl RemoteNodeResponse {
    /// Constructs a successful response.
    #[must_use]
    pub fn success(
        request: &RemoteNodeRequest,
        responder_cluster_id: impl Into<String>,
        responder_agent_id: impl Into<String>,
        output: QianjiOutput,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            node_id: request.node_id.clone(),
            responder_cluster_id: responder_cluster_id.into(),
            responder_agent_id: responder_agent_id.into(),
            ok: true,
            output: Some(output),
            error: None,
            finished_ms: current_unix_millis(),
        }
    }

    /// Constructs a failed response.
    #[must_use]
    pub fn failure(
        request: &RemoteNodeRequest,
        responder_cluster_id: impl Into<String>,
        responder_agent_id: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            node_id: request.node_id.clone(),
            responder_cluster_id: responder_cluster_id.into(),
            responder_agent_id: responder_agent_id.into(),
            ok: false,
            output: None,
            error: Some(error.into()),
            finished_ms: current_unix_millis(),
        }
    }
}

/// Valkey transport for remote possession request/response orchestration.
pub struct RemotePossessionBus {
    redis_url: String,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl RemotePossessionBus {
    /// Creates a new possession bus from Valkey URL.
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    fn request_key(request_id: &str) -> String {
        format!("xiuxian:swarm:possession:req:{request_id}")
    }

    fn response_key(request_id: &str) -> String {
        format!("xiuxian:swarm:possession:resp:{request_id}")
    }

    fn queue_key(role_class: &str) -> String {
        format!(
            "xiuxian:swarm:possession:queue:{}",
            role_class.trim().to_ascii_lowercase()
        )
    }

    fn response_channel(request_id: &str) -> String {
        format!("xiuxian:swarm:possession:channel:{request_id}")
    }

    /// Submits a remote request and enqueues it for target role workers.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or Valkey commands fail.
    pub async fn submit_request(
        &self,
        request: &RemoteNodeRequest,
        ttl_seconds: u64,
    ) -> Result<()> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        if request.role_class.trim().is_empty() {
            return Err(anyhow!("request.role_class must not be empty"));
        }
        let request_json = serde_json::to_string(request)?;
        let req_key = Self::request_key(&request.request_id);
        let queue_key = Self::queue_key(&request.role_class);

        let _: i64 = self
            .run_command("remote_possession_hset_request", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&req_key)
                    .arg("request")
                    .arg(&request_json)
                    .arg("status")
                    .arg("pending")
                    .arg("created_ms")
                    .arg(request.created_ms.to_string());
                command
            })
            .await?;
        let _: bool = self
            .run_command("remote_possession_expire_request", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&req_key).arg(ttl_seconds);
                command
            })
            .await?;
        let _: i64 = self
            .run_command("remote_possession_queue_push", || {
                let mut command = redis::cmd("RPUSH");
                command.arg(&queue_key).arg(&request.request_id);
                command
            })
            .await?;
        Ok(())
    }

    /// Claims one pending request from a role queue.
    ///
    /// Returns `Ok(None)` when no request arrives in `block_timeout`.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or request payload is malformed.
    pub async fn claim_next_for_role(
        &self,
        role_class: &str,
        claimer_id: &str,
        block_timeout: Duration,
    ) -> Result<Option<RemoteNodeRequest>> {
        let queue_key = Self::queue_key(role_class);
        let timeout_seconds = block_timeout.as_secs().max(1);
        let popped: Option<Vec<String>> = self
            .run_command("remote_possession_blpop", || {
                let mut command = redis::cmd("BLPOP");
                command.arg(&queue_key).arg(timeout_seconds);
                command
            })
            .await?;

        let Some(raw) = popped else {
            return Ok(None);
        };
        if raw.len() != 2 {
            return Ok(None);
        }
        let request_id = raw[1].clone();
        let req_key = Self::request_key(&request_id);
        let request_json: Option<String> = self
            .run_command("remote_possession_hget_request_payload", || {
                let mut command = redis::cmd("HGET");
                command.arg(&req_key).arg("request");
                command
            })
            .await?;
        let Some(request_json) = request_json else {
            return Ok(None);
        };
        let request: RemoteNodeRequest = serde_json::from_str(&request_json)?;
        let _: i64 = self
            .run_command("remote_possession_hset_claimed", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&req_key)
                    .arg("status")
                    .arg("claimed")
                    .arg("claimer_id")
                    .arg(claimer_id);
                command
            })
            .await?;
        Ok(Some(request))
    }

    /// Publishes one response for a previously submitted request.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or Valkey commands fail.
    pub async fn submit_response(
        &self,
        response: &RemoteNodeResponse,
        ttl_seconds: u64,
    ) -> Result<()> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        let response_json = serde_json::to_string(response)?;
        let response_key = Self::response_key(&response.request_id);
        let request_key = Self::request_key(&response.request_id);
        let channel = Self::response_channel(&response.request_id);

        let _: () = self
            .run_command("remote_possession_set_response", || {
                let mut command = redis::cmd("SET");
                command.arg(&response_key).arg(&response_json);
                command
            })
            .await?;
        let _: bool = self
            .run_command("remote_possession_expire_response", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&response_key).arg(ttl_seconds);
                command
            })
            .await?;
        let _: i64 = self
            .run_command("remote_possession_hset_request_done", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&request_key)
                    .arg("status")
                    .arg("completed")
                    .arg("finished_ms")
                    .arg(response.finished_ms.to_string());
                command
            })
            .await?;
        let _: i64 = self
            .run_command("remote_possession_publish_response", || {
                let mut command = redis::cmd("PUBLISH");
                command.arg(&channel).arg(&response_json);
                command
            })
            .await?;
        Ok(())
    }

    /// Waits for response of one request.
    ///
    /// Returns `Ok(None)` on timeout.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey/pubsub operations fail.
    pub async fn wait_response(
        &self,
        request_id: &str,
        max_wait: Duration,
    ) -> Result<Option<RemoteNodeResponse>> {
        let response_key = Self::response_key(request_id);
        let existing: Option<String> = self
            .run_command("remote_possession_get_response", || {
                let mut command = redis::cmd("GET");
                command.arg(&response_key);
                command
            })
            .await?;
        if let Some(payload) = existing {
            let parsed = serde_json::from_str::<RemoteNodeResponse>(&payload)?;
            return Ok(Some(parsed));
        }

        let client = redis::Client::open(self.redis_url.as_str())
            .context("Failed to open Valkey connection for remote possession wait")?;
        let mut pubsub = client.get_async_pubsub().await?;
        let channel = Self::response_channel(request_id);
        pubsub.subscribe(channel).await?;
        let mut stream = pubsub.on_message();

        match timeout(max_wait, async {
            while let Some(message) = stream.next().await {
                let payload: String = message.get_payload()?;
                let parsed = serde_json::from_str::<RemoteNodeResponse>(&payload)?;
                if parsed.request_id == request_id {
                    return Ok(parsed);
                }
            }
            Err(anyhow!(
                "remote possession pubsub stream closed unexpectedly"
            ))
        })
        .await
        {
            Ok(inner) => inner.map(Some),
            Err(_elapsed) => Ok(None),
        }
    }

    /// Convenience helper: submit request and wait for one response.
    ///
    /// # Errors
    ///
    /// Returns an error when submit or wait operations fail.
    pub async fn request_and_wait(
        &self,
        request: &RemoteNodeRequest,
        ttl_seconds: u64,
        max_wait: Duration,
    ) -> Result<Option<RemoteNodeResponse>> {
        self.submit_request(request, ttl_seconds).await?;
        self.wait_response(&request.request_id, max_wait).await
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
            .context("Failed to connect to Valkey for remote possession")?;
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

fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

/// Converts any mechanism error into failed remote response.
#[must_use]
pub fn map_execution_error_to_response(
    request: &RemoteNodeRequest,
    responder_cluster_id: &str,
    responder_agent_id: &str,
    error: &str,
) -> RemoteNodeResponse {
    RemoteNodeResponse {
        request_id: request.request_id.clone(),
        session_id: request.session_id.clone(),
        node_id: request.node_id.clone(),
        responder_cluster_id: responder_cluster_id.to_string(),
        responder_agent_id: responder_agent_id.to_string(),
        ok: false,
        output: Some(QianjiOutput {
            data: serde_json::json!({
                "remote_possession_error": error,
            }),
            instruction: FlowInstruction::Abort(error.to_string()),
        }),
        error: Some(error.to_string()),
        finished_ms: current_unix_millis(),
    }
}
