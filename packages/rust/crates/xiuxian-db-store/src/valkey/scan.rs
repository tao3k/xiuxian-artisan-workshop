//! Client retention and deadline admission for complete cursor enumeration.

use std::{collections::HashSet, num::NonZeroUsize, time::Duration};

use super::{ValkeyClient, ValkeyStoreError};

/// Limits retained key payloads, not transport buffers or server work.
#[derive(Clone, Copy, Debug)]
pub struct ValkeyScanPolicy {
    /// Maximum logical SCAN calls, excluding transport retries.
    pub max_calls: NonZeroUsize,
    /// Maximum distinct retained keys.
    pub max_keys: NonZeroUsize,
    /// Maximum sum of retained UTF-8 key lengths.
    pub max_key_bytes: NonZeroUsize,
    /// Deadline covering connection acquisition and all scan pages.
    pub timeout: Duration,
}

impl Default for ValkeyScanPolicy {
    fn default() -> Self {
        Self {
            max_calls: NonZeroUsize::new(1024).unwrap_or(NonZeroUsize::MIN),
            max_keys: NonZeroUsize::new(100_000).unwrap_or(NonZeroUsize::MIN),
            max_key_bytes: NonZeroUsize::new(8 * 1024 * 1024).unwrap_or(NonZeroUsize::MIN),
            timeout: Duration::from_secs(5),
        }
    }
}

struct RetainedKeys {
    keys: HashSet<String>,
    bytes: usize,
}

impl RetainedKeys {
    fn admit(&mut self, key: String, policy: ValkeyScanPolicy) -> Result<(), ValkeyStoreError> {
        if self.keys.contains(&key) {
            return Ok(());
        }
        if self.keys.len() >= policy.max_keys.get() {
            return Err(exhausted("keys"));
        }
        let bytes = self
            .bytes
            .checked_add(key.len())
            .ok_or_else(|| exhausted("key_bytes"))?;
        if bytes > policy.max_key_bytes.get() {
            return Err(exhausted("key_bytes"));
        }
        self.keys.insert(key);
        self.bytes = bytes;
        Ok(())
    }
}

fn exhausted(resource: &'static str) -> ValkeyStoreError {
    ValkeyStoreError::ScanBudgetExceeded { resource }
}

impl ValkeyClient {
    /// Returns only complete, deduplicated enumeration results in unspecified order.
    /// COUNT is a hint; oversized and empty nonterminal pages are valid.
    ///
    /// # Errors
    /// Returns a typed budget error without partial results on exhaustion, or a
    /// storage error when the backend fails. Zero duration expires immediately.
    pub async fn scan_keys_with_policy(
        &self,
        pattern: &str,
        policy: ValkeyScanPolicy,
    ) -> Result<Vec<String>, ValkeyStoreError> {
        collect_pages(policy, |cursor| async move {
            self.run_read_command("valkey_scan_keys", || {
                let mut command = redis::cmd("SCAN");
                command
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(256);
                command
            })
            .await
        })
        .await
    }
}

async fn collect_pages<F, Fut>(
    policy: ValkeyScanPolicy,
    mut fetch: F,
) -> Result<Vec<String>, ValkeyStoreError>
where
    F: FnMut(u64) -> Fut,
    Fut: std::future::Future<Output = Result<(u64, Vec<String>), ValkeyStoreError>>,
{
    if policy.timeout.is_zero() {
        return Err(exhausted("deadline"));
    }
    let started = tokio::time::Instant::now();
    tokio::time::timeout(policy.timeout, async {
        let mut retained = RetainedKeys {
            keys: HashSet::new(),
            bytes: 0,
        };
        let mut cursor = 0_u64;
        for _ in 0..policy.max_calls.get() {
            let (next, page) = fetch(cursor).await?;
            for (index, key) in page.into_iter().enumerate() {
                if index % 256 == 0 {
                    tokio::task::yield_now().await;
                    if started.elapsed() >= policy.timeout {
                        return Err(exhausted("deadline"));
                    }
                }
                retained.admit(key, policy)?;
            }
            if started.elapsed() >= policy.timeout {
                return Err(exhausted("deadline"));
            }
            if next == 0 {
                return Ok(retained.keys.into_iter().collect());
            }
            cursor = next;
            tokio::task::yield_now().await;
        }
        Err(exhausted("calls"))
    })
    .await
    .map_err(|_| exhausted("deadline"))?
}

#[cfg(test)]
#[path = "../../tests/unit/valkey_scan.rs"]
mod tests;
