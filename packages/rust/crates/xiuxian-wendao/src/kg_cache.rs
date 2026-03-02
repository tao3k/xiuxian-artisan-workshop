//! In-process cache for `KnowledgeGraph` loaded from Valkey.
//!
//! Avoids repeated backend reads when the same graph scope key is accessed
//! across multiple recall operations. Cache is invalidated on save so ingest
//! updates are visible.

use crate::graph::{GraphError, KnowledgeGraph};
use log::debug;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

static KG_CACHE: LazyLock<Mutex<HashMap<String, KnowledgeGraph>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Normalize path/scope for cache key.
///
/// Keeps historical path behavior stable while supporting logical scope keys.
fn normalize_scope(scope_key: &str) -> String {
    let trimmed = scope_key.trim_end_matches('/');
    Path::new(trimmed).canonicalize().map_or_else(
        |_| trimmed.to_string(),
        |p| p.to_string_lossy().into_owned(),
    )
}

/// Load `KnowledgeGraph` from Valkey, using in-process cache when available.
///
/// On cache hit, returns a clone of cached graph.
/// On cache miss, loads from backend, stores cache, returns clone.
///
/// # Errors
///
/// Returns [`GraphError::InvalidRelation`] when lock or runtime initialization fails.
pub fn load_from_valkey_cached(scope_key: &str) -> Result<Option<KnowledgeGraph>, GraphError> {
    let key = normalize_scope(scope_key);

    {
        let cache = KG_CACHE
            .lock()
            .map_err(|e| GraphError::InvalidRelation("cache_lock".into(), e.to_string()))?;
        if let Some(cached) = cache.get(&key) {
            debug!("KG cache hit for scope: {key}");
            return Ok(Some(cached.clone()));
        }
    }

    let graph = load_from_valkey_impl(scope_key)?;
    let result = if graph.get_stats().total_entities == 0 && graph.get_stats().total_relations == 0
    {
        Some(graph)
    } else {
        let cloned = graph.clone();
        {
            let mut cache = KG_CACHE
                .lock()
                .map_err(|e| GraphError::InvalidRelation("cache_lock".into(), e.to_string()))?;
            cache.insert(key.clone(), graph);
            debug!(
                "KG cache insert for scope: {key} ({} entities, {} relations)",
                cloned.get_stats().total_entities,
                cloned.get_stats().total_relations
            );
        }
        Some(cloned)
    };

    Ok(result)
}

fn load_from_valkey_impl(scope_key: &str) -> Result<KnowledgeGraph, GraphError> {
    let mut graph = KnowledgeGraph::new();
    graph.load_from_valkey_sync(scope_key)?;
    Ok(graph)
}

/// Invalidate cache for the given scope key.
pub fn invalidate(scope_key: &str) {
    let key = normalize_scope(scope_key);
    if let Ok(mut cache) = KG_CACHE.lock()
        && cache.remove(&key).is_some()
    {
        debug!("KG cache invalidated for scope: {key}");
    }
}

/// Invalidate all cached graphs (for testing or full reset).
pub fn invalidate_all() {
    if let Ok(mut cache) = KG_CACHE.lock() {
        let count = cache.len();
        cache.clear();
        if count > 0 {
            debug!("KG cache cleared: {count} entries");
        }
    }
}

/// Return the number of cached entries (for testing).
#[must_use]
pub fn cache_len() -> usize {
    KG_CACHE.lock().map_or(0, |c| c.len())
}
