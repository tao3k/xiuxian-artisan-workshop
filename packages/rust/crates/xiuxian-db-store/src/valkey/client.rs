//! Shared multiplexed Valkey client execution and reconnect handling.

use std::sync::Arc;

use redis::FromRedisValue;
use tokio::sync::{Mutex, RwLock};

use crate::valkey::{
    ValkeyStoreConfig,
    error::{ValkeyStoreError, validate_positive_ttl},
};

/// Shared multiplexed Valkey client.
#[derive(Clone)]
pub struct ValkeyClient {
    config: ValkeyStoreConfig,
    connection: Arc<RwLock<Option<Arc<redis::aio::MultiplexedConnection>>>>,
    reconnect_lock: Arc<Mutex<()>>,
}

impl ValkeyClient {
    /// Creates a client.
    #[must_use]
    pub fn new(config: ValkeyStoreConfig) -> Self {
        Self {
            config,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Borrows the config.
    #[must_use]
    pub const fn config(&self) -> &ValkeyStoreConfig {
        &self.config
    }

    pub(crate) async fn run_command<T, F>(
        &self,
        operation: &'static str,
        build: F,
    ) -> Result<T, ValkeyStoreError>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        self.execute(operation, build, false).await
    }

    pub(crate) async fn run_read_command<T, F>(
        &self,
        operation: &'static str,
        build: F,
    ) -> Result<T, ValkeyStoreError>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        self.execute(operation, build, true).await
    }

    async fn execute<T, F>(
        &self,
        operation: &'static str,
        build: F,
        replay_read: bool,
    ) -> Result<T, ValkeyStoreError>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_error: Option<redis::RedisError> = None;
        for _ in 0..if replay_read { 2 } else { 1 } {
            let generation = self.acquire_connection().await?;
            let mut connection = (*generation).clone();
            let command = build();
            let result: redis::RedisResult<T> = command.query_async(&mut connection).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    if error.is_io_error() {
                        self.invalidate_connection(&generation).await;
                    }
                    if !replay_read {
                        return Err(ValkeyStoreError::OutcomeUnknown {
                            operation,
                            message: error.to_string(),
                        });
                    }
                    if !error.is_io_error() {
                        return Err(ValkeyStoreError::Storage {
                            operation,
                            message: error.to_string(),
                        });
                    }
                    last_error = Some(error);
                }
            }
        }
        Err(ValkeyStoreError::Storage {
            operation,
            message: last_error.map_or_else(
                || "Valkey command failed unexpectedly".to_owned(),
                |error| error.to_string(),
            ),
        })
    }

    async fn acquire_connection(
        &self,
    ) -> Result<Arc<redis::aio::MultiplexedConnection>, ValkeyStoreError> {
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let _guard = self.reconnect_lock.lock().await;
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let client = redis::Client::open(self.config.redis_url()).map_err(|error| {
            ValkeyStoreError::Storage {
                operation: "open_valkey_client",
                message: error.to_string(),
            }
        })?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| ValkeyStoreError::Storage {
                operation: "connect_valkey",
                message: error.to_string(),
            })?;
        let connection = Arc::new(connection);
        {
            let mut guard = self.connection.write().await;
            *guard = Some(connection.clone());
        }
        Ok(connection)
    }

    async fn invalidate_connection(&self, failed: &Arc<redis::aio::MultiplexedConnection>) {
        let mut guard = self.connection.write().await;
        if guard
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, failed))
        {
            *guard = None;
        }
    }

    /// Stores a string with a millisecond TTL.
    ///
    /// # Errors
    ///
    /// Returns an error when `ttl_ms` is zero or the Valkey command fails.
    pub async fn set_string_with_ttl(
        &self,
        key: &str,
        value: &str,
        ttl_ms: u64,
    ) -> Result<(), ValkeyStoreError> {
        validate_positive_ttl("ttl_ms", ttl_ms)?;
        let _: String = self
            .run_command("valkey_set_string_with_ttl", || {
                let mut command = redis::cmd("SET");
                command.arg(key).arg(value).arg("PX").arg(ttl_ms);
                command
            })
            .await?;
        Ok(())
    }

    /// Loads a string value.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey command fails.
    pub async fn get_string(&self, key: &str) -> Result<Option<String>, ValkeyStoreError> {
        self.run_read_command("valkey_get_string", || {
            let mut command = redis::cmd("GET");
            command.arg(key);
            command
        })
        .await
    }

    /// Completes an incremental scan using the default client resource policy.
    ///
    /// # Errors
    ///
    /// Returns an error on storage failure or incomplete enumeration.
    pub async fn scan_keys(&self, pattern: &str) -> Result<Vec<String>, ValkeyStoreError> {
        self.scan_keys_with_policy(pattern, super::ValkeyScanPolicy::default())
            .await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/valkey_client.rs"]
mod tests;
