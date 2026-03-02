use chrono::Utc;

use crate::types::KnowledgeEntry;

use super::KnowledgeStorage;

impl KnowledgeStorage {
    /// Initialize the storage (validate Valkey connectivity).
    ///
    /// # Errors
    ///
    /// Returns an error when `Valkey` connectivity check fails.
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.redis_connection().await?;
        let _pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    }

    /// Upsert a knowledge entry.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or `Valkey` operations fail.
    pub async fn upsert(&self, entry: &KnowledgeEntry) -> Result<(), Box<dyn std::error::Error>> {
        self.init().await?;
        let mut conn = self.redis_connection().await?;
        let entries_key = self.entries_key();
        let existing_raw: Option<String> = redis::cmd("HGET")
            .arg(&entries_key)
            .arg(&entry.id)
            .query_async(&mut conn)
            .await?;
        let existing = existing_raw
            .as_deref()
            .map(serde_json::from_str::<KnowledgeEntry>)
            .transpose()?;

        let now = Utc::now();
        let (created_at, version) = if let Some(found) = existing {
            (found.created_at, found.version + 1)
        } else {
            (now, entry.version.max(1))
        };

        let mut to_store = entry.clone();
        to_store.created_at = created_at;
        to_store.updated_at = now;
        to_store.version = version;
        let payload = serde_json::to_string(&to_store)?;

        let _: i64 = redis::cmd("HSET")
            .arg(entries_key)
            .arg(&to_store.id)
            .arg(payload)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    /// Count total entries.
    ///
    /// # Errors
    ///
    /// Returns an error when `Valkey` operations fail.
    pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let mut conn = self.redis_connection().await?;
        let total: i64 = redis::cmd("HLEN")
            .arg(self.entries_key())
            .query_async(&mut conn)
            .await?;
        Ok(total)
    }

    /// Delete an entry by ID.
    ///
    /// # Errors
    ///
    /// Returns an error when `Valkey` operations fail.
    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.redis_connection().await?;
        let _: i64 = redis::cmd("HDEL")
            .arg(self.entries_key())
            .arg(id)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    /// Clear all entries.
    ///
    /// # Errors
    ///
    /// Returns an error when `Valkey` operations fail.
    pub async fn clear(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.redis_connection().await?;
        let _: i64 = redis::cmd("DEL")
            .arg(self.entries_key())
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}
