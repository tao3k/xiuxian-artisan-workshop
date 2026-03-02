use anyhow::{Context, Result, bail};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

use crate::observability::SessionEvent;

use super::token::next_lease_owner_token;
use super::{DEFAULT_GATE_RETRY_INTERVAL_MS, DistributedLeaseGuard, ValkeySessionGateBackend};

impl ValkeySessionGateBackend {
    pub(in crate::channels::telegram::session_gate) fn new(
        valkey_url: &str,
        key_prefix: &str,
        lease_ttl_secs: u64,
        acquire_timeout_secs: Option<u64>,
    ) -> Result<Self> {
        let client = redis::Client::open(valkey_url).with_context(|| {
            format!("invalid valkey url for session gate backend: {valkey_url}")
        })?;
        Ok(Self {
            client,
            key_prefix: key_prefix.to_string(),
            lease_ttl_ms: lease_ttl_secs.saturating_mul(1000),
            acquire_timeout: acquire_timeout_secs
                .filter(|value| *value > 0)
                .map(Duration::from_secs),
            retry_interval: Duration::from_millis(DEFAULT_GATE_RETRY_INTERVAL_MS),
            connection: Arc::new(tokio::sync::RwLock::new(None)),
            reconnect_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    pub(in crate::channels::telegram::session_gate) async fn acquire_lease(
        self: &Arc<Self>,
        session_id: &str,
    ) -> Result<DistributedLeaseGuard> {
        let lock_key = format!("{}:lock:{}", self.key_prefix, session_id);
        let owner_token = next_lease_owner_token(session_id);
        let started = Instant::now();
        loop {
            if self.try_acquire_lease(&lock_key, &owner_token).await? {
                break;
            }

            if let Some(timeout) = self.acquire_timeout
                && started.elapsed() >= timeout
            {
                tracing::warn!(
                    event = SessionEvent::SessionGateLeaseAcquireTimeout.as_str(),
                    session_id,
                    wait_ms = started.elapsed().as_millis(),
                    timeout_ms = timeout.as_millis(),
                    "timed out waiting for distributed session gate lease"
                );
                bail!(
                    "timed out waiting {}ms for distributed session gate lease",
                    timeout.as_millis()
                );
            }
            tokio::time::sleep(self.retry_interval).await;
        }

        let wait_ms = started.elapsed().as_millis();
        tracing::debug!(
            event = SessionEvent::SessionGateLeaseAcquired.as_str(),
            session_id,
            wait_ms,
            lease_ttl_ms = self.lease_ttl_ms,
            "distributed session gate lease acquired"
        );

        let (stop_tx, stop_rx) = oneshot::channel::<()>();
        self.spawn_lease_renew_task(lock_key.clone(), owner_token.clone(), stop_rx);
        Ok(DistributedLeaseGuard {
            backend: Arc::clone(self),
            lock_key,
            owner_token,
            stop_tx: Some(stop_tx),
        })
    }

    fn spawn_lease_renew_task(
        self: &Arc<Self>,
        lock_key: String,
        owner_token: String,
        mut stop_rx: oneshot::Receiver<()>,
    ) {
        let backend = Arc::clone(self);
        let renew_interval_ms = (backend.lease_ttl_ms / 3).max(200);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(renew_interval_ms));
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = ticker.tick() => {
                        match backend.renew_lease(&lock_key, &owner_token).await {
                            Ok(true) => {
                                tracing::debug!(
                                    event = SessionEvent::SessionGateLeaseRenewed.as_str(),
                                    key = %lock_key,
                                    renew_interval_ms,
                                    "distributed session gate lease renewed"
                                );
                            }
                            Ok(false) => {
                                tracing::warn!(
                                    event = SessionEvent::SessionGateLeaseRenewalFailed.as_str(),
                                    key = %lock_key,
                                    "distributed session gate lease lost before renewal"
                                );
                                break;
                            }
                            Err(error) => {
                                tracing::warn!(
                                    event = SessionEvent::SessionGateLeaseRenewalFailed.as_str(),
                                    key = %lock_key,
                                    error = %error,
                                    "distributed session gate lease renewal failed"
                                );
                            }
                        }
                    }
                }
            }
        });
    }
}
