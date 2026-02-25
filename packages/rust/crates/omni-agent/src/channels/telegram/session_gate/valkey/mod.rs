use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
use tokio::sync::{Mutex, oneshot};

pub(super) const DEFAULT_GATE_RETRY_INTERVAL_MS: u64 = 25;

static NEXT_LEASE_OWNER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(super) struct ValkeySessionGateBackend {
    client: redis::Client,
    key_prefix: String,
    lease_ttl_ms: u64,
    acquire_timeout: Option<Duration>,
    retry_interval: Duration,
    connection: Arc<Mutex<Option<redis::aio::MultiplexedConnection>>>,
}

pub(super) struct DistributedLeaseGuard {
    backend: Arc<ValkeySessionGateBackend>,
    lock_key: String,
    owner_token: String,
    stop_tx: Option<oneshot::Sender<()>>,
}

mod acquire;
mod commands;
mod guard;
mod token;
