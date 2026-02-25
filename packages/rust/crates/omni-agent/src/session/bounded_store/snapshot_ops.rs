use anyhow::{Context, Result};

use super::BoundedSessionStore;

impl BoundedSessionStore {
    /// Atomically reset active bounded-session state into backup keys.
    ///
    /// # Errors
    /// Returns an error when the underlying Valkey atomic reset operation fails.
    pub async fn atomic_reset_snapshot(
        &self,
        session_id: &str,
        backup_session_id: &str,
        metadata_session_id: &str,
        saved_at_unix_ms: u64,
    ) -> Result<Option<(usize, usize)>> {
        let Some(ref redis) = self.redis else {
            return Ok(None);
        };
        let stats = redis
            .atomic_reset_bounded_snapshot(
                session_id,
                backup_session_id,
                metadata_session_id,
                saved_at_unix_ms,
            )
            .await
            .with_context(|| {
                format!("atomic bounded snapshot reset failed for session_id={session_id}")
            })?;
        Ok(Some(stats))
    }

    /// Atomically restore active bounded-session state from backup keys.
    ///
    /// # Errors
    /// Returns an error when the underlying Valkey atomic resume operation fails.
    pub async fn atomic_resume_snapshot(
        &self,
        session_id: &str,
        backup_session_id: &str,
        metadata_session_id: &str,
    ) -> Result<Option<(usize, usize)>> {
        let Some(ref redis) = self.redis else {
            return Ok(None);
        };
        redis
            .atomic_resume_bounded_snapshot(session_id, backup_session_id, metadata_session_id)
            .await
            .with_context(|| {
                format!("atomic bounded snapshot resume failed for session_id={session_id}")
            })
    }

    /// Atomically delete bounded-session backup keys.
    ///
    /// # Errors
    /// Returns an error when the underlying Valkey atomic drop operation fails.
    pub async fn atomic_drop_snapshot(
        &self,
        backup_session_id: &str,
        metadata_session_id: &str,
    ) -> Result<Option<bool>> {
        let Some(ref redis) = self.redis else {
            return Ok(None);
        };
        let dropped = redis
            .atomic_drop_bounded_snapshot(backup_session_id, metadata_session_id)
            .await
            .with_context(|| {
                format!(
                    "atomic bounded snapshot drop failed for backup_session_id={backup_session_id}"
                )
            })?;
        Ok(Some(dropped))
    }
}
