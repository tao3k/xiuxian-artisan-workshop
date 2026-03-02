use std::process;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use redis::FromRedisValue;
use tokio::sync::{Mutex, RwLock};
use xiuxian_zhenfa::{
    ZhenfaAuditSink, ZhenfaDispatchEvent, ZhenfaDispatchOutcome, ZhenfaError, ZhenfaMutationGuard,
    ZhenfaMutationLock, ZhenfaOrchestratorHooks, ZhenfaResultCache,
};

use crate::config::XiuxianConfig;

const DEFAULT_ZHENFA_VALKEY_KEY_PREFIX: &str = "omni:zhenfa";
const DEFAULT_ZHENFA_VALKEY_CACHE_TTL_SECONDS: u64 = 120;
const DEFAULT_ZHENFA_VALKEY_LOCK_TTL_SECONDS: u64 = 15;
const DEFAULT_ZHENFA_VALKEY_AUDIT_STREAM: &str = "dispatch.audit";
const LOCK_BUSY_ERROR_CODE: i32 = -32011;

const RELEASE_LOCK_IF_OWNER_SCRIPT: &str = r#"
if redis.call("GET", KEYS[1]) == ARGV[1] then
  return redis.call("DEL", KEYS[1])
end
return 0
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ZhenfaValkeyHookConfig {
    pub(super) url: String,
    pub(super) key_prefix: String,
    pub(super) cache_ttl_seconds: u64,
    pub(super) lock_ttl_seconds: u64,
    pub(super) audit_stream: String,
}

#[derive(Clone)]
struct ZhenfaValkeyHookRuntime {
    client: redis::Client,
    key_prefix: String,
    cache_ttl_seconds: u64,
    lock_ttl_seconds: u64,
    audit_stream: String,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl ZhenfaValkeyHookRuntime {
    async fn run_command<T, F>(&self, operation: &'static str, build: F) -> Result<T, ZhenfaError>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_error: Option<redis::RedisError> = None;
        for attempt in 0..2 {
            let mut connection = self.acquire_connection().await?;
            let command = build();
            let result: redis::RedisResult<T> = command.query_async(&mut connection).await;
            match result {
                Ok(value) => {
                    if attempt > 0 {
                        tracing::debug!(
                            event = "zhenfa.valkey.command.retry_succeeded",
                            operation,
                            attempt = attempt + 1,
                            "zhenfa valkey command succeeded after retry"
                        );
                    }
                    return Ok(value);
                }
                Err(error) => {
                    tracing::warn!(
                        event = "zhenfa.valkey.command.retry_failed",
                        operation,
                        attempt = attempt + 1,
                        error = %error,
                        "zhenfa valkey command failed; reconnecting"
                    );
                    self.invalidate_connection().await;
                    last_error = Some(error);
                }
            }
        }
        let message = last_error.map_or_else(
            || format!("zhenfa valkey command `{operation}` failed unexpectedly"),
            |error| format!("zhenfa valkey command `{operation}` failed: {error}"),
        );
        Err(ZhenfaError::execution(message))
    }

    async fn acquire_connection(&self) -> Result<redis::aio::MultiplexedConnection, ZhenfaError> {
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
            .map_err(|error| {
                ZhenfaError::execution(format!("failed to open zhenfa valkey connection: {error}"))
            })?;
        {
            let mut guard = self.connection.write().await;
            *guard = Some(connection.clone());
        }
        tracing::debug!(
            event = "zhenfa.valkey.connected",
            key_prefix = %self.key_prefix,
            "zhenfa valkey hooks connected"
        );
        Ok(connection)
    }

    async fn invalidate_connection(&self) {
        let mut guard = self.connection.write().await;
        *guard = None;
    }

    fn cache_key(&self, cache_key: &str) -> String {
        format!("{}:cache:{cache_key}", self.key_prefix)
    }

    fn lock_key(&self, mutation_scope: &str) -> String {
        format!("{}:lock:{mutation_scope}", self.key_prefix)
    }

    fn stream_key(&self) -> String {
        format!("{}:stream:{}", self.key_prefix, self.audit_stream)
    }

    async fn release_lock_if_owner(&self, key: String, token: String) -> Result<(), ZhenfaError> {
        let _: i64 = self
            .run_command("release_lock_if_owner", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RELEASE_LOCK_IF_OWNER_SCRIPT)
                    .arg(1)
                    .arg(&key)
                    .arg(&token);
                command
            })
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
struct ZhenfaValkeyResultCache {
    runtime: Arc<ZhenfaValkeyHookRuntime>,
}

#[async_trait]
impl ZhenfaResultCache for ZhenfaValkeyResultCache {
    async fn get(&self, key: &str) -> Result<Option<String>, ZhenfaError> {
        let cache_key = self.runtime.cache_key(key);
        self.runtime
            .run_command("cache_get", || {
                let mut command = redis::cmd("GET");
                command.arg(&cache_key);
                command
            })
            .await
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), ZhenfaError> {
        let cache_key = self.runtime.cache_key(key);
        let ttl_seconds = self.runtime.cache_ttl_seconds;
        let _: String = self
            .runtime
            .run_command("cache_set", || {
                let mut command = redis::cmd("SET");
                command
                    .arg(&cache_key)
                    .arg(value)
                    .arg("EX")
                    .arg(ttl_seconds);
                command
            })
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
struct ZhenfaValkeyMutationLock {
    runtime: Arc<ZhenfaValkeyHookRuntime>,
}

#[async_trait]
impl ZhenfaMutationLock for ZhenfaValkeyMutationLock {
    async fn acquire(&self, scope: &str) -> Result<Box<dyn ZhenfaMutationGuard>, ZhenfaError> {
        let lock_key = self.runtime.lock_key(scope);
        let lock_token = build_lock_token();
        let ttl_seconds = self.runtime.lock_ttl_seconds;
        let set_result: Option<String> = self
            .runtime
            .run_command("lock_acquire", || {
                let mut command = redis::cmd("SET");
                command
                    .arg(&lock_key)
                    .arg(&lock_token)
                    .arg("EX")
                    .arg(ttl_seconds)
                    .arg("NX");
                command
            })
            .await?;
        if set_result.is_none() {
            return Err(ZhenfaError::execution_with_code(
                LOCK_BUSY_ERROR_CODE,
                format!("zhenfa mutation lock busy: {scope}"),
            ));
        }
        Ok(Box::new(ZhenfaValkeyMutationGuard {
            runtime: Arc::clone(&self.runtime),
            key: lock_key,
            token: lock_token,
        }))
    }
}

struct ZhenfaValkeyMutationGuard {
    runtime: Arc<ZhenfaValkeyHookRuntime>,
    key: String,
    token: String,
}

impl ZhenfaMutationGuard for ZhenfaValkeyMutationGuard {}

impl Drop for ZhenfaValkeyMutationGuard {
    fn drop(&mut self) {
        let runtime = Arc::clone(&self.runtime);
        let key = self.key.clone();
        let token = self.token.clone();
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                if let Err(error) = runtime.release_lock_if_owner(key, token).await {
                    tracing::warn!(
                        event = "zhenfa.valkey.lock.release_failed",
                        error = %error,
                        "zhenfa valkey lock release failed"
                    );
                }
            });
        }
    }
}

#[derive(Clone)]
struct ZhenfaValkeyAuditSink {
    runtime: Arc<ZhenfaValkeyHookRuntime>,
}

#[async_trait]
impl ZhenfaAuditSink for ZhenfaValkeyAuditSink {
    async fn emit(&self, event: ZhenfaDispatchEvent) -> Result<(), ZhenfaError> {
        let stream_key = self.runtime.stream_key();
        let fields = vec![
            ("tool_id".to_string(), event.tool_id),
            (
                "session_id".to_string(),
                event.session_id.unwrap_or_default(),
            ),
            ("trace_id".to_string(), event.trace_id.unwrap_or_default()),
            ("elapsed_ms".to_string(), event.elapsed_ms.to_string()),
            ("outcome".to_string(), dispatch_outcome_name(&event.outcome)),
            (
                "error_code".to_string(),
                event
                    .error_code
                    .map_or_else(String::new, |code| code.to_string()),
            ),
            (
                "error_message".to_string(),
                event.error_message.unwrap_or_default(),
            ),
            ("ts_ms".to_string(), unix_timestamp_millis().to_string()),
        ];
        let _: String = self
            .runtime
            .run_command("audit_emit", || {
                let mut command = redis::cmd("XADD");
                command.arg(&stream_key).arg("*");
                for (key, value) in &fields {
                    command.arg(key).arg(value);
                }
                command
            })
            .await?;
        Ok(())
    }
}

#[must_use]
pub(super) fn build_zhenfa_orchestrator_hooks(
    config: &XiuxianConfig,
) -> Option<ZhenfaOrchestratorHooks> {
    let runtime_config = resolve_zhenfa_valkey_hook_config(config)?;
    let client = match redis::Client::open(runtime_config.url.as_str()) {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!(
                event = "zhenfa.valkey.disabled.invalid_url",
                url = %runtime_config.url,
                error = %error,
                "zhenfa valkey hooks disabled because URL is invalid"
            );
            return None;
        }
    };
    let runtime = Arc::new(ZhenfaValkeyHookRuntime {
        client,
        key_prefix: runtime_config.key_prefix,
        cache_ttl_seconds: runtime_config.cache_ttl_seconds,
        lock_ttl_seconds: runtime_config.lock_ttl_seconds,
        audit_stream: runtime_config.audit_stream,
        connection: Arc::new(RwLock::new(None)),
        reconnect_lock: Arc::new(Mutex::new(())),
    });
    Some(ZhenfaOrchestratorHooks {
        cache: Some(Arc::new(ZhenfaValkeyResultCache {
            runtime: Arc::clone(&runtime),
        })),
        mutation_lock: Some(Arc::new(ZhenfaValkeyMutationLock {
            runtime: Arc::clone(&runtime),
        })),
        audit_sink: Some(Arc::new(ZhenfaValkeyAuditSink { runtime })),
        signal_sink: None,
    })
}

#[must_use]
pub(super) fn resolve_zhenfa_valkey_hook_config(
    config: &XiuxianConfig,
) -> Option<ZhenfaValkeyHookConfig> {
    let url = non_empty(config.zhenfa.valkey.url.as_deref())?;
    Some(ZhenfaValkeyHookConfig {
        url,
        key_prefix: non_empty(config.zhenfa.valkey.key_prefix.as_deref())
            .unwrap_or_else(|| DEFAULT_ZHENFA_VALKEY_KEY_PREFIX.to_string()),
        cache_ttl_seconds: config
            .zhenfa
            .valkey
            .cache_ttl_seconds
            .unwrap_or(DEFAULT_ZHENFA_VALKEY_CACHE_TTL_SECONDS)
            .max(1),
        lock_ttl_seconds: config
            .zhenfa
            .valkey
            .lock_ttl_seconds
            .unwrap_or(DEFAULT_ZHENFA_VALKEY_LOCK_TTL_SECONDS)
            .max(1),
        audit_stream: non_empty(config.zhenfa.valkey.audit_stream.as_deref())
            .unwrap_or_else(|| DEFAULT_ZHENFA_VALKEY_AUDIT_STREAM.to_string()),
    })
}

#[must_use]
fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|trimmed| !trimmed.is_empty())
        .map(str::to_string)
}

#[must_use]
fn dispatch_outcome_name(outcome: &ZhenfaDispatchOutcome) -> String {
    match outcome {
        ZhenfaDispatchOutcome::Success => "success".to_string(),
        ZhenfaDispatchOutcome::Cached => "cached".to_string(),
        ZhenfaDispatchOutcome::Failed => "failed".to_string(),
    }
}

#[must_use]
fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis())
}

#[must_use]
fn build_lock_token() -> String {
    let ts_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("pid:{}:ts_ns:{ts_ns}", process::id())
}
