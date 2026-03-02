//! Integration tests for KG cache (`load_from_valkey_cached`, `invalidate`).
//!
//! Tests share a static cache; run with:
//! `cargo test -p xiuxian-wendao kg_cache -- --test-threads=1`.

use std::sync::{LazyLock, Mutex, MutexGuard};
use tempfile::TempDir;
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_wendao::kg_cache::{cache_len, invalidate, invalidate_all, load_from_valkey_cached};
use xiuxian_wendao::{Entity, EntityType};

static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn test_guard() -> MutexGuard<'static, ()> {
    TEST_LOCK
        .lock()
        .unwrap_or_else(|_| panic!("kg cache test lock poisoned"))
}

fn has_valkey() -> bool {
    std::env::var("VALKEY_URL")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

fn create_test_kg_with_entity() -> Result<(TempDir, String), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let scope_key = tmp.path().join("kg").to_string_lossy().into_owned();

    let graph = KnowledgeGraph::new();
    let entity = Entity::new(
        "test:foo".to_string(),
        "Foo".to_string(),
        EntityType::Concept,
        "Test entity".to_string(),
    );
    assert!(graph.add_entity(entity).is_ok());
    graph.save_to_valkey(&scope_key, 8)?;
    Ok((tmp, scope_key))
}

fn load_cached_required(scope_key: &str) -> Result<KnowledgeGraph, Box<dyn std::error::Error>> {
    let graph = load_from_valkey_cached(scope_key)?
        .ok_or_else(|| std::io::Error::other("cached load should return a graph"))?;
    Ok(graph)
}

#[test]
fn test_cache_miss_then_hit() -> Result<(), Box<dyn std::error::Error>> {
    if !has_valkey() {
        return Ok(());
    }
    let _guard = test_guard();
    invalidate_all();
    let (_tmp, scope_key) = create_test_kg_with_entity()?;

    let g1 = load_cached_required(&scope_key)?;
    assert_eq!(g1.get_stats().total_entities, 1);

    let g2 = load_cached_required(&scope_key)?;
    assert_eq!(g2.get_stats().total_entities, 1);
    assert_eq!(cache_len(), 1, "cache should have one entry");
    Ok(())
}

#[test]
fn test_cache_invalidation_after_save() -> Result<(), Box<dyn std::error::Error>> {
    if !has_valkey() {
        return Ok(());
    }
    let _guard = test_guard();
    invalidate_all();
    let (_tmp, scope_key) = create_test_kg_with_entity()?;

    let g1 = load_cached_required(&scope_key)?;
    assert_eq!(g1.get_stats().total_entities, 1);
    assert_eq!(cache_len(), 1);

    invalidate(&scope_key);
    assert_eq!(cache_len(), 0, "invalidate should remove the entry");

    let g2 = load_cached_required(&scope_key)?;
    assert_eq!(g2.get_stats().total_entities, 1);
    Ok(())
}

#[test]
fn test_nonexistent_path_returns_empty() -> Result<(), Box<dyn std::error::Error>> {
    if !has_valkey() {
        return Ok(());
    }
    let _guard = test_guard();
    invalidate_all();
    let result = load_from_valkey_cached("nonexistent.scope")?;
    assert!(result.is_some());
    let g = result.ok_or_else(|| std::io::Error::other("expected empty graph instance"))?;
    assert_eq!(g.get_stats().total_entities, 0);
    assert_eq!(g.get_stats().total_relations, 0);
    assert_eq!(cache_len(), 0);
    Ok(())
}

#[test]
fn test_path_normalization() -> Result<(), Box<dyn std::error::Error>> {
    if !has_valkey() {
        return Ok(());
    }
    let _guard = test_guard();
    invalidate_all();
    let (_tmp, scope_key) = create_test_kg_with_entity()?;
    let scope_key_trailing = format!("{scope_key}/");

    let g1 = load_cached_required(&scope_key)?;
    let g2 = load_cached_required(&scope_key_trailing)?;
    assert_eq!(g1.get_stats().total_entities, g2.get_stats().total_entities);
    assert_eq!(
        cache_len(),
        1,
        "normalized paths should share one cache entry"
    );
    Ok(())
}
