use std::{collections::HashSet, sync::Arc};

use redis::FromRedisValue;

use super::{ReadBudget, ValkeyReadPolicy, ValkeyReadUsage};
use crate::valkey::{ValkeyClient, ValkeyQueueEntryId, ValkeyQueueKeys, ValkeyStoreError};

/// Read-only capability sharing one admission budget across cloned components.
#[derive(Clone)]
pub struct ValkeyReadSession {
    client: ValkeyClient,
    budget: Arc<ReadBudget>,
}

impl ValkeyReadSession {
    /// Starts an observation without opening a connection.
    #[must_use]
    pub fn new(client: ValkeyClient, policy: ValkeyReadPolicy) -> Self {
        Self {
            client,
            budget: Arc::new(ReadBudget::new(policy)),
        }
    }

    /// Checks final admission after caller-side assembly and returns usage.
    ///
    /// # Errors
    /// Returns terminal budget exhaustion, including an elapsed deadline.
    pub fn finish(&self) -> Result<ValkeyReadUsage, ValkeyStoreError> {
        self.budget.admit(0, 0, 0)
    }

    async fn query<T: FromRedisValue + Send>(
        &self,
        operation: &'static str,
        command: redis::Cmd,
    ) -> Result<T, ValkeyStoreError> {
        self.budget
            .within(
                self.client
                    .run_budgeted_read(operation, || command.clone(), &self.budget),
            )
            .await
    }

    /// Reads a bounded pending index, with one overflow sentinel.
    ///
    /// # Errors
    /// Returns exhaustion instead of a truncated successful list, or backend failure.
    pub async fn pending_entries(
        &self,
        keys: &ValkeyQueueKeys,
    ) -> Result<Vec<ValkeyQueueEntryId>, ValkeyStoreError> {
        self.entries(keys.pending_key()).await
    }

    /// Reads a bounded lease index, with one overflow sentinel.
    ///
    /// # Errors
    /// Returns exhaustion instead of a truncated successful list, or backend failure.
    pub async fn lease_entries(
        &self,
        keys: &ValkeyQueueKeys,
    ) -> Result<Vec<ValkeyQueueEntryId>, ValkeyStoreError> {
        self.entries(keys.lease_deadlines_key()).await
    }

    async fn entries(&self, key: &str) -> Result<Vec<ValkeyQueueEntryId>, ValkeyStoreError> {
        let remaining = self.budget.remaining_items()?;
        let mut command = redis::cmd("ZRANGE");
        // Inclusive stop: one extra entry detects truncation, even at zero allowance.
        command.arg(key).arg(0).arg(remaining);
        let entries: Vec<String> = self.query("observation_queue_entries", command).await?;
        self.budget.admit(0, entries.len(), 0)?;
        entries
            .into_iter()
            .map(|entry| {
                self.budget.admit(0, 0, entry.len())?;
                ValkeyQueueEntryId::new(entry)
            })
            .collect()
    }

    /// Enumerates distinct heartbeat keys until cursor zero under the shared budget.
    ///
    /// # Errors
    /// Returns exhaustion without partial success, or a backend error.
    pub async fn scan_keys(&self, pattern: &str) -> Result<Vec<String>, ValkeyStoreError> {
        let mut keys = HashSet::new();
        let mut cursor = 0_u64;
        loop {
            let mut command = redis::cmd("SCAN");
            command
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(256);
            let (next, page): (u64, Vec<String>) = self.query("observation_scan", command).await?;
            for (index, key) in page.into_iter().enumerate() {
                if index % 256 == 0 {
                    tokio::task::yield_now().await;
                }
                if !keys.contains(&key) {
                    self.budget.admit(0, 1, key.len())?;
                    keys.insert(key);
                }
            }
            self.finish()?;
            if next == 0 {
                return Ok(keys.into_iter().collect());
            }
            cursor = next;
        }
    }

    /// Reads one bounded raw payload from a queue.
    ///
    /// # Errors
    /// Returns budget exhaustion or backend failure.
    pub async fn payload(
        &self,
        keys: &ValkeyQueueKeys,
        entry: &ValkeyQueueEntryId,
    ) -> Result<Option<String>, ValkeyStoreError> {
        let mut command = redis::cmd("HGET");
        command.arg(keys.payload_key(entry)).arg("payload");
        self.string_query(command).await
    }

    /// Reads one heartbeat payload.
    ///
    /// # Errors
    /// Returns budget exhaustion or backend failure.
    pub async fn string(&self, key: &str) -> Result<Option<String>, ValkeyStoreError> {
        let mut command = redis::cmd("GET");
        command.arg(key);
        self.string_query(command).await
    }

    async fn string_query(&self, command: redis::Cmd) -> Result<Option<String>, ValkeyStoreError> {
        let value: Option<String> = self.query("observation_payload", command).await?;
        self.budget
            .admit(0, 0, value.as_ref().map_or(0, String::len))?;
        Ok(value)
    }

    /// Reads lease fields under the same byte and command budget.
    ///
    /// # Errors
    /// Returns budget exhaustion or backend failure.
    pub async fn lease(
        &self,
        keys: &ValkeyQueueKeys,
        entry: &ValkeyQueueEntryId,
    ) -> Result<Vec<(String, String)>, ValkeyStoreError> {
        let mut command = redis::cmd("HGETALL");
        command.arg(keys.lease_key(entry));
        let fields: Vec<(String, String)> = self.query("observation_lease", command).await?;
        for (key, value) in &fields {
            self.budget.admit(0, 0, key.len())?;
            self.budget.admit(0, 0, value.len())?;
        }
        Ok(fields)
    }
}
