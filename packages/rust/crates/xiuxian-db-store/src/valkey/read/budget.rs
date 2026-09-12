use std::{sync::Mutex, time::Duration};

use crate::valkey::ValkeyStoreError;

/// Limits one complete observation, including read transport retries.
/// Zero budgets deny the corresponding resource. Bytes count admitted raw data,
/// not wire buffers, decoded object overhead, or server work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValkeyReadPolicy {
    /// Maximum actual command submissions across all components.
    pub max_commands: usize,
    /// Maximum enumerated queue entries and distinct heartbeat keys combined.
    pub max_items: usize,
    /// Maximum cumulative admitted key, payload, and lease field bytes.
    pub max_bytes: usize,
    /// Shared deadline from request construction through final assembly.
    pub timeout: Duration,
}

impl Default for ValkeyReadPolicy {
    fn default() -> Self {
        Self {
            max_commands: 20_000,
            max_items: 10_000,
            max_bytes: 8 * 1024 * 1024,
            timeout: Duration::from_secs(5),
        }
    }
}

/// Admitted work receipt, not process memory or latency measurement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ValkeyReadUsage {
    /// Actual submissions, including retries.
    pub commands: usize,
    /// Enumerated entries and distinct keys.
    pub items: usize,
    /// Cumulative admitted raw data bytes.
    pub bytes: usize,
}

#[derive(Default)]
struct State {
    usage: ValkeyReadUsage,
    failure: Option<&'static str>,
}

pub(crate) struct ReadBudget {
    pub policy: ValkeyReadPolicy,
    pub started: tokio::time::Instant,
    state: Mutex<State>,
}

impl ReadBudget {
    pub fn new(policy: ValkeyReadPolicy) -> Self {
        Self {
            policy,
            started: tokio::time::Instant::now(),
            state: Mutex::default(),
        }
    }

    pub fn admit(
        &self,
        commands: usize,
        items: usize,
        bytes: usize,
    ) -> Result<ValkeyReadUsage, ValkeyStoreError> {
        let mut state = self.state.lock().map_err(|_| ValkeyStoreError::Storage {
            operation: "read_budget",
            message: "budget lock poisoned".into(),
        })?;
        let failure = state.failure.or_else(|| {
            if self.started.elapsed() >= self.policy.timeout {
                Some("deadline")
            } else if state
                .usage
                .commands
                .checked_add(commands)
                .is_none_or(|n| n > self.policy.max_commands)
            {
                Some("commands")
            } else if state
                .usage
                .items
                .checked_add(items)
                .is_none_or(|n| n > self.policy.max_items)
            {
                Some("items")
            } else if state
                .usage
                .bytes
                .checked_add(bytes)
                .is_none_or(|n| n > self.policy.max_bytes)
            {
                Some("bytes")
            } else {
                None
            }
        });
        if let Some(resource) = failure {
            state.failure = Some(resource);
            return Err(ValkeyStoreError::ReadBudgetExceeded { resource });
        }
        state.usage.commands += commands;
        state.usage.items += items;
        state.usage.bytes += bytes;
        Ok(state.usage)
    }

    pub fn remaining_items(&self) -> Result<usize, ValkeyStoreError> {
        Ok(self.policy.max_items - self.admit(0, 0, 0)?.items)
    }

    pub async fn within<T>(
        &self,
        future: impl std::future::Future<Output = Result<T, ValkeyStoreError>>,
    ) -> Result<T, ValkeyStoreError> {
        self.admit(0, 0, 0)?;
        let remaining = self.policy.timeout.saturating_sub(self.started.elapsed());
        if let Ok(result) = tokio::time::timeout(remaining, future).await {
            self.admit(0, 0, 0)?;
            result
        } else {
            // Record terminal exhaustion so concurrent components cannot finish successfully.
            self.admit(0, 0, 0)?;
            Err(ValkeyStoreError::ReadBudgetExceeded {
                resource: "deadline",
            })
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/valkey_read_budget.rs"]
mod tests;
