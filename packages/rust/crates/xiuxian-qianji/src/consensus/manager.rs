use super::models::{AgentIdentity, AgentVote, ConsensusPolicy, ConsensusResult};
use super::thresholds::required_weight_threshold;
use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use redis::FromRedisValue;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, timeout};

const CONSENSUS_VOTE_TTL_SECONDS: u64 = 300;

#[derive(Debug, Clone)]
struct VoteKeys {
    base: String,
    votes_hash: String,
    weight_counter: String,
    winner_marker: String,
    first_seen_marker: String,
    output_payloads: String,
}

impl VoteKeys {
    fn new(session_id: &str, node_id: &str) -> Self {
        let base = format!("xiuxian:consensus:{session_id}:{node_id}");
        Self {
            base: base.clone(),
            votes_hash: format!("{base}:votes"),
            weight_counter: format!("{base}:counts"),
            winner_marker: format!("{base}:winner"),
            first_seen_marker: format!("{base}:first_seen_ms"),
            output_payloads: format!("{base}:outputs"),
        }
    }

    fn quorum_channel(&self) -> String {
        format!("{}:channel", self.base)
    }
}

#[derive(Debug, Clone, Copy)]
struct VoteSnapshot {
    total_agents: usize,
    hash_weight: f64,
}

/// Orchestrates distributed voting via Valkey.
pub struct ConsensusManager {
    redis_url: String,
    agent_identity: AgentIdentity,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl ConsensusManager {
    /// Creates a consensus manager backed by the given Valkey/Redis URL.
    ///
    /// Agent identity defaults to:
    /// - `AGENT_ID` env or `local_agent`
    /// - `AGENT_WEIGHT` env or `1.0`
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self::with_agent_identity(redis_url, AgentIdentity::from_env())
    }

    /// Creates a consensus manager with explicit agent identity.
    #[must_use]
    pub fn with_agent_identity(redis_url: String, agent_identity: AgentIdentity) -> Self {
        Self {
            redis_url,
            agent_identity,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Submits one vote and returns consensus verdict in one Rust-side flow.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or vote serialization fails.
    pub async fn submit_vote(
        &self,
        session_id: &str,
        node_id: &str,
        output_hash: String,
        policy: &ConsensusPolicy,
    ) -> Result<ConsensusResult> {
        self.submit_vote_with_payload(session_id, node_id, output_hash, None, policy)
            .await
    }

    /// Submits one vote with optional serialized output payload.
    ///
    /// This method is used by scheduler-level consensus gates so that non-winning
    /// agent processes can materialize the agreed payload without recomputation.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or vote serialization fails.
    pub async fn submit_vote_with_payload(
        &self,
        session_id: &str,
        node_id: &str,
        output_hash: String,
        output_payload: Option<&str>,
        policy: &ConsensusPolicy,
    ) -> Result<ConsensusResult> {
        let vote = AgentVote {
            agent_id: self.agent_identity.id.clone(),
            output_hash,
            weight: self.agent_identity.weight,
            timestamp_ms: u128::from(current_unix_millis()),
        };
        self.submit_vote_payload(session_id, node_id, vote, output_payload, policy)
            .await
    }

    /// Returns the stored output payload for an agreed hash, if available.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey lookup fails.
    pub async fn get_output_payload(
        &self,
        session_id: &str,
        node_id: &str,
        output_hash: &str,
    ) -> Result<Option<String>> {
        let keys = VoteKeys::new(session_id, node_id);
        self.run_command("consensus_get_output_payload", || {
            let mut command = redis::cmd("HGET");
            command.arg(&keys.output_payloads).arg(output_hash);
            command
        })
        .await
    }

    /// Waits asynchronously for quorum result published for one node.
    ///
    /// Returns `Ok(Some(hash))` when winner hash is observed, `Ok(None)` on timeout.
    ///
    /// # Errors
    ///
    /// Returns an error when pub/sub connection or channel operations fail.
    pub async fn wait_for_quorum(
        &self,
        session_id: &str,
        node_id: &str,
        max_wait: Duration,
    ) -> Result<Option<String>> {
        let keys = VoteKeys::new(session_id, node_id);
        if let Some(winner) = self.current_winner(keys.winner_marker.as_str()).await? {
            return Ok(Some(winner));
        }

        let client = redis::Client::open(self.redis_url.as_str())
            .context("Failed to connect to Valkey pubsub for consensus wait")?;
        let mut pubsub = client.get_async_pubsub().await?;
        let channel = keys.quorum_channel();
        pubsub.subscribe(channel).await?;
        let mut stream = pubsub.on_message();

        match timeout(max_wait, async {
            while let Some(message) = stream.next().await {
                let payload: String = message.get_payload()?;
                if !payload.trim().is_empty() {
                    return Ok(payload);
                }
            }
            Err(anyhow!("consensus pubsub stream closed unexpectedly"))
        })
        .await
        {
            Ok(inner) => inner.map(Some),
            Err(_elapsed) => Ok(None),
        }
    }

    async fn submit_vote_payload(
        &self,
        session_id: &str,
        node_id: &str,
        vote: AgentVote,
        output_payload: Option<&str>,
        policy: &ConsensusPolicy,
    ) -> Result<ConsensusResult> {
        let keys = VoteKeys::new(session_id, node_id);
        if let Some(winner) = self.current_winner(keys.winner_marker.as_str()).await? {
            return Ok(ConsensusResult::Agreed(winner));
        }

        if let Some(payload) = output_payload {
            self.store_output_payload(&keys, vote.output_hash.as_str(), payload)
                .await?;
        }

        let snapshot = self.record_vote(keys.clone(), &vote).await?;
        let required_weight = required_weight_threshold(policy, snapshot.total_agents);
        if snapshot.total_agents >= policy.min_agents && snapshot.hash_weight >= required_weight {
            let winner = self
                .mark_or_read_winner(&keys, vote.output_hash.as_str())
                .await?;
            return Ok(ConsensusResult::Agreed(winner));
        }

        if self
            .timeout_exceeded(keys.first_seen_marker.as_str(), policy)
            .await?
        {
            return Ok(ConsensusResult::Failed("consensus_timeout".to_string()));
        }

        Ok(ConsensusResult::Pending)
    }

    async fn record_vote(&self, keys: VoteKeys, vote: &AgentVote) -> Result<VoteSnapshot> {
        let payload = serde_json::to_string(vote)?;

        let _: i64 = self
            .run_command("consensus_store_vote_payload", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&keys.votes_hash)
                    .arg(&vote.agent_id)
                    .arg(&payload);
                command
            })
            .await?;

        let new_weight: f64 = self
            .run_command("consensus_increment_hash_weight", || {
                let mut command = redis::cmd("HINCRBYFLOAT");
                command
                    .arg(&keys.weight_counter)
                    .arg(&vote.output_hash)
                    .arg(vote.weight);
                command
            })
            .await?;

        let total_agents: usize = self
            .run_command("consensus_read_total_agents", || {
                let mut command = redis::cmd("HLEN");
                command.arg(&keys.votes_hash);
                command
            })
            .await?;

        let vote_ts = u64::try_from(vote.timestamp_ms).unwrap_or(u64::MAX);
        let _: i64 = self
            .run_command("consensus_set_first_seen_if_absent", || {
                let mut command = redis::cmd("SETNX");
                command.arg(&keys.first_seen_marker).arg(vote_ts);
                command
            })
            .await?;

        self.refresh_ttls(&keys).await?;

        Ok(VoteSnapshot {
            total_agents,
            hash_weight: new_weight,
        })
    }

    async fn refresh_ttls(&self, keys: &VoteKeys) -> Result<()> {
        let ttl = CONSENSUS_VOTE_TTL_SECONDS;
        let _: bool = self
            .run_command("consensus_expire_votes", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&keys.votes_hash).arg(ttl);
                command
            })
            .await?;
        let _: bool = self
            .run_command("consensus_expire_counts", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&keys.weight_counter).arg(ttl);
                command
            })
            .await?;
        let _: bool = self
            .run_command("consensus_expire_first_seen", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&keys.first_seen_marker).arg(ttl);
                command
            })
            .await?;
        let _: bool = self
            .run_command("consensus_expire_winner", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&keys.winner_marker).arg(ttl);
                command
            })
            .await?;
        let _: bool = self
            .run_command("consensus_expire_outputs", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&keys.output_payloads).arg(ttl);
                command
            })
            .await?;
        Ok(())
    }

    async fn current_winner(&self, winner_key: &str) -> Result<Option<String>> {
        self.run_command("consensus_get_winner", || {
            let mut command = redis::cmd("GET");
            command.arg(winner_key);
            command
        })
        .await
    }

    async fn mark_or_read_winner(&self, keys: &VoteKeys, hash: &str) -> Result<String> {
        let was_inserted: i64 = self
            .run_command("consensus_set_winner_nx", || {
                let mut command = redis::cmd("SETNX");
                command.arg(&keys.winner_marker).arg(hash);
                command
            })
            .await?;
        if was_inserted == 1 {
            self.publish_winner(keys, hash).await?;
        }
        if let Some(winner) = self.current_winner(keys.winner_marker.as_str()).await? {
            return Ok(winner);
        }
        Ok(hash.to_string())
    }

    async fn store_output_payload(
        &self,
        keys: &VoteKeys,
        output_hash: &str,
        payload: &str,
    ) -> Result<()> {
        let _: i64 = self
            .run_command("consensus_store_output_payload", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&keys.output_payloads)
                    .arg(output_hash)
                    .arg(payload);
                command
            })
            .await?;
        Ok(())
    }

    async fn publish_winner(&self, keys: &VoteKeys, hash: &str) -> Result<()> {
        let channel = keys.quorum_channel();
        let _: i64 = self
            .run_command("consensus_publish_winner", || {
                let mut command = redis::cmd("PUBLISH");
                command.arg(&channel).arg(hash);
                command
            })
            .await?;
        Ok(())
    }

    async fn timeout_exceeded(
        &self,
        first_seen_key: &str,
        policy: &ConsensusPolicy,
    ) -> Result<bool> {
        if policy.timeout_ms == 0 {
            return Ok(false);
        }
        let first_seen_ms: Option<u64> = self
            .run_command("consensus_get_first_seen_ms", || {
                let mut command = redis::cmd("GET");
                command.arg(first_seen_key);
                command
            })
            .await?;
        let Some(first_seen_ms) = first_seen_ms else {
            return Ok(false);
        };
        let now_ms = current_unix_millis();
        Ok(now_ms.saturating_sub(first_seen_ms) >= policy.timeout_ms)
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
            .context("Failed to connect to Valkey for consensus")?;
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
