//! Top-level integration harness for `agent::memory_recall_state`.

mod config {
    pub(crate) use omni_agent::AgentConfig;
}

mod session {
    use std::collections::HashMap;
    use std::sync::Arc;

    use anyhow::Result;
    use tokio::sync::RwLock;

    pub(crate) use omni_agent::ChatMessage;

    #[derive(Clone)]
    struct RedisConfig {
        url: String,
        key_prefix: String,
        ttl_secs: Option<u64>,
    }

    /// Lightweight session store shim with in-memory and optional Valkey modes.
    pub(crate) struct SessionStore {
        memory: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
        redis: Option<RedisConfig>,
    }

    impl SessionStore {
        pub(crate) fn new() -> Result<Self> {
            Ok(Self {
                memory: Arc::new(RwLock::new(HashMap::new())),
                redis: None,
            })
        }

        pub(crate) fn new_with_redis(
            redis_url: impl Into<String>,
            key_prefix: Option<String>,
            ttl_secs: Option<u64>,
        ) -> Result<Self> {
            Ok(Self {
                memory: Arc::new(RwLock::new(HashMap::new())),
                redis: Some(RedisConfig {
                    url: redis_url.into(),
                    key_prefix: key_prefix.unwrap_or_else(|| "omni-agent".to_string()),
                    ttl_secs,
                }),
            })
        }

        fn session_key(redis: &RedisConfig, session_id: &str) -> String {
            format!("{}:session:{session_id}", redis.key_prefix)
        }

        fn stream_key(redis: &RedisConfig, stream_name: &str) -> String {
            format!("{}:stream:{stream_name}", redis.key_prefix)
        }

        fn metrics_key(redis: &RedisConfig, stream_name: &str, session_id: Option<&str>) -> String {
            match session_id {
                Some(session_id) if !session_id.trim().is_empty() => {
                    format!(
                        "{}:metrics:{stream_name}:session:{}",
                        redis.key_prefix,
                        session_id.trim()
                    )
                }
                _ => format!("{}:metrics:{stream_name}", redis.key_prefix),
            }
        }

        pub(crate) async fn append(
            &self,
            session_id: &str,
            messages: Vec<ChatMessage>,
        ) -> Result<()> {
            if messages.is_empty() {
                return Ok(());
            }
            if let Some(redis) = self.redis.as_ref() {
                let client = redis::Client::open(redis.url.as_str())?;
                let mut conn = client.get_multiplexed_async_connection().await?;
                let key = Self::session_key(redis, session_id);
                for message in messages {
                    let payload = serde_json::to_string(&message)?;
                    let _: usize = redis::cmd("RPUSH")
                        .arg(&key)
                        .arg(payload)
                        .query_async(&mut conn)
                        .await?;
                }
                if let Some(ttl) = redis.ttl_secs {
                    let _: bool = redis::cmd("EXPIRE")
                        .arg(&key)
                        .arg(ttl)
                        .query_async(&mut conn)
                        .await?;
                }
                return Ok(());
            }
            let mut guard = self.memory.write().await;
            guard
                .entry(session_id.to_string())
                .or_default()
                .extend(messages);
            Ok(())
        }

        pub(crate) async fn get(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
            if let Some(redis) = self.redis.as_ref() {
                let client = redis::Client::open(redis.url.as_str())?;
                let mut conn = client.get_multiplexed_async_connection().await?;
                let key = Self::session_key(redis, session_id);
                let payloads: Vec<String> = redis::cmd("LRANGE")
                    .arg(&key)
                    .arg(0)
                    .arg(-1)
                    .query_async(&mut conn)
                    .await?;
                let mut messages = Vec::with_capacity(payloads.len());
                for payload in payloads {
                    let message = serde_json::from_str::<ChatMessage>(&payload)?;
                    messages.push(message);
                }
                return Ok(messages);
            }
            let guard = self.memory.read().await;
            Ok(guard.get(session_id).cloned().unwrap_or_default())
        }

        pub(crate) async fn replace(
            &self,
            session_id: &str,
            messages: Vec<ChatMessage>,
        ) -> Result<()> {
            if let Some(redis) = self.redis.as_ref() {
                let client = redis::Client::open(redis.url.as_str())?;
                let mut conn = client.get_multiplexed_async_connection().await?;
                let key = Self::session_key(redis, session_id);
                let _: i64 = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
                if !messages.is_empty() {
                    for message in messages {
                        let payload = serde_json::to_string(&message)?;
                        let _: usize = redis::cmd("RPUSH")
                            .arg(&key)
                            .arg(payload)
                            .query_async(&mut conn)
                            .await?;
                    }
                    if let Some(ttl) = redis.ttl_secs {
                        let _: bool = redis::cmd("EXPIRE")
                            .arg(&key)
                            .arg(ttl)
                            .query_async(&mut conn)
                            .await?;
                    }
                }
                return Ok(());
            }
            let mut guard = self.memory.write().await;
            if messages.is_empty() {
                guard.remove(session_id);
            } else {
                guard.insert(session_id.to_string(), messages);
            }
            Ok(())
        }

        pub(crate) async fn clear(&self, session_id: &str) -> Result<()> {
            if let Some(redis) = self.redis.as_ref() {
                let client = redis::Client::open(redis.url.as_str())?;
                let mut conn = client.get_multiplexed_async_connection().await?;
                let key = Self::session_key(redis, session_id);
                let _: i64 = redis::cmd("DEL").arg(&key).query_async(&mut conn).await?;
                return Ok(());
            }
            let mut guard = self.memory.write().await;
            guard.remove(session_id);
            Ok(())
        }

        pub(crate) async fn publish_stream_event(
            &self,
            stream_name: &str,
            fields: Vec<(String, String)>,
        ) -> Result<Option<String>> {
            let Some(redis) = self.redis.as_ref() else {
                return Ok(None);
            };
            let client = redis::Client::open(redis.url.as_str())?;
            let mut conn = client.get_multiplexed_async_connection().await?;

            let stream_key = Self::stream_key(redis, stream_name);
            let mut command = redis::cmd("XADD");
            command.arg(&stream_key).arg("*");
            for (key, value) in &fields {
                command.arg(key).arg(value);
            }
            let event_id: String = command.query_async(&mut conn).await?;

            let event_kind = fields
                .iter()
                .find_map(|(key, value)| (key == "kind").then_some(value.as_str()))
                .unwrap_or("unknown")
                .to_string();
            let session_id = fields
                .iter()
                .find_map(|(key, value)| (key == "session_id").then_some(value.as_str()));

            let global_metrics_key = Self::metrics_key(redis, stream_name, None);
            let _: i64 = redis::cmd("HINCRBY")
                .arg(&global_metrics_key)
                .arg("events_total")
                .arg(1)
                .query_async(&mut conn)
                .await?;
            let _: i64 = redis::cmd("HINCRBY")
                .arg(&global_metrics_key)
                .arg(format!("kind:{event_kind}"))
                .arg(1)
                .query_async(&mut conn)
                .await?;

            if let Some(session_id) = session_id {
                let scoped_metrics_key = Self::metrics_key(redis, stream_name, Some(session_id));
                let _: i64 = redis::cmd("HINCRBY")
                    .arg(&scoped_metrics_key)
                    .arg("events_total")
                    .arg(1)
                    .query_async(&mut conn)
                    .await?;
                let _: i64 = redis::cmd("HINCRBY")
                    .arg(&scoped_metrics_key)
                    .arg(format!("kind:{event_kind}"))
                    .arg(1)
                    .query_async(&mut conn)
                    .await?;
            }

            Ok(Some(event_id))
        }
    }
}

mod agent {
    pub(crate) struct Agent {
        pub(crate) session: crate::session::SessionStore,
        memory_stream_name: String,
    }

    impl Agent {
        pub(crate) async fn from_config(
            _config: crate::config::AgentConfig,
        ) -> anyhow::Result<Self> {
            Ok(Self {
                session: crate::session::SessionStore::new()?,
                memory_stream_name: "memory.events".to_string(),
            })
        }

        pub(crate) async fn from_config_with_session_backends_for_test(
            _config: crate::config::AgentConfig,
            session: crate::session::SessionStore,
            _bounded_session: Option<()>,
        ) -> anyhow::Result<Self> {
            Ok(Self {
                session,
                memory_stream_name: "memory.events".to_string(),
            })
        }

        pub(crate) fn memory_stream_name(&self) -> &str {
            &self.memory_stream_name
        }
    }

    pub(crate) mod memory_recall {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub(crate) struct MemoryRecallPlan {
            pub(crate) k1: usize,
            pub(crate) k2: usize,
            pub(crate) lambda: f32,
            pub(crate) min_score: f32,
            pub(crate) max_context_chars: usize,
            pub(crate) budget_pressure: f32,
            pub(crate) window_pressure: f32,
            pub(crate) effective_budget_tokens: Option<usize>,
        }
    }

    pub(crate) mod memory_recall_state {
        include!("../src/agent/memory_recall_state/mod.rs");

        fn lint_symbol_probe() {
            let plan = crate::agent::memory_recall::MemoryRecallPlan {
                k1: 1,
                k2: 1,
                lambda: 0.5,
                min_score: 0.1,
                max_context_chars: 256,
                budget_pressure: 0.2,
                window_pressure: 0.3,
                effective_budget_tokens: Some(100),
            };
            let _ = crate::agent::memory_recall_state::SessionMemoryRecallSnapshot::from_plan;
            let _ = plan.effective_budget_tokens;
            let _ = std::mem::size_of::<SessionMemoryRecallSnapshotInput>();
        }

        const _: fn() = lint_symbol_probe;

        mod tests {
            include!("agent/memory_recall_state.rs");
        }
    }
}
