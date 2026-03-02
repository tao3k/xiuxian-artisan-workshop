use crate::types::KnowledgeEntry;
use xxhash_rust::xxh3::xxh3_64;

use super::{
    DEFAULT_KNOWLEDGE_VALKEY_KEY_PREFIX, KNOWLEDGE_VALKEY_KEY_PREFIX_ENV, KNOWLEDGE_VALKEY_URL_ENV,
    KnowledgeStorage,
};

impl KnowledgeStorage {
    pub(super) fn resolve_valkey_url() -> Result<String, Box<dyn std::error::Error>> {
        let url = std::env::var(KNOWLEDGE_VALKEY_URL_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                std::env::var("VALKEY_URL")
                    .ok()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
            })
            .ok_or_else(|| {
                format!(
                    "knowledge storage valkey url is required (set {KNOWLEDGE_VALKEY_URL_ENV} or VALKEY_URL)"
                )
            })?;
        Ok(url)
    }

    pub(super) fn resolve_key_prefix(&self) -> String {
        let configured = std::env::var(KNOWLEDGE_VALKEY_KEY_PREFIX_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_KNOWLEDGE_VALKEY_KEY_PREFIX.to_string());
        let scope = format!("{}|{}", self.path.to_string_lossy(), self.table_name);
        let hash = xxh3_64(scope.as_bytes());
        format!("{configured}:{hash:016x}")
    }

    pub(super) fn entries_key(&self) -> String {
        format!("{}:entries", self.resolve_key_prefix())
    }

    pub(super) fn redis_client() -> Result<redis::Client, Box<dyn std::error::Error>> {
        let url = Self::resolve_valkey_url()?;
        Ok(redis::Client::open(url.as_str())?)
    }

    pub(super) async fn redis_connection(
        &self,
    ) -> Result<redis::aio::MultiplexedConnection, Box<dyn std::error::Error>> {
        let client = Self::redis_client()?;
        Ok(client.get_multiplexed_async_connection().await?)
    }

    pub(super) async fn load_all_entries(
        &self,
    ) -> Result<Vec<KnowledgeEntry>, Box<dyn std::error::Error>> {
        let mut conn = self.redis_connection().await?;
        let raw_entries: Vec<String> = redis::cmd("HVALS")
            .arg(self.entries_key())
            .query_async(&mut conn)
            .await?;
        raw_entries
            .into_iter()
            .map(|raw| {
                serde_json::from_str::<KnowledgeEntry>(&raw).map_err(|error| {
                    let boxed: Box<dyn std::error::Error> = Box::new(error);
                    boxed
                })
            })
            .collect()
    }
}
