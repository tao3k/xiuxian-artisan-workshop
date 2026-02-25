use std::sync::Arc;

use crate::observability::SessionEvent;

use super::DistributedLeaseGuard;

impl Drop for DistributedLeaseGuard {
    fn drop(&mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let backend = Arc::clone(&self.backend);
        let lock_key = self.lock_key.clone();
        let owner_token = self.owner_token.clone();
        handle.spawn(async move {
            match backend.release_lease(&lock_key, &owner_token).await {
                Ok(released) => {
                    tracing::debug!(
                        event = SessionEvent::SessionGateLeaseReleased.as_str(),
                        key = %lock_key,
                        released,
                        "valkey session gate lease release attempted"
                    );
                }
                Err(error) => {
                    tracing::warn!(
                        event = SessionEvent::SessionGateLeaseReleased.as_str(),
                        key = %lock_key,
                        error = %error,
                        "valkey session gate lease release failed"
                    );
                }
            }
        });
    }
}
