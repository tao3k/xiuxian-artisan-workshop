//! Integration tests for `xiuxian_wendao::storage`.

use tempfile::TempDir;
use xiuxian_wendao::{KnowledgeCategory, KnowledgeEntry, KnowledgeStorage};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

fn configure_test_valkey() -> bool {
    if let Ok(url) = std::env::var("VALKEY_URL")
        && !url.trim().is_empty()
    {
        return true;
    }
    false
}

fn text_to_vector(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0_f32; 128];
    for (idx, byte) in text.as_bytes().iter().enumerate() {
        let bucket = idx % 128;
        vec[bucket] += f32::from(*byte) / 255.0;
    }
    let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vec {
            *value /= norm;
        }
    }
    vec
}

#[tokio::test]
async fn test_storage_creation() -> TestResult {
    let temp_dir = TempDir::new()?;
    let storage = KnowledgeStorage::new(temp_dir.path().to_string_lossy().as_ref(), "knowledge");

    assert_eq!(storage.table_name(), "knowledge");
    Ok(())
}

#[tokio::test]
async fn test_upsert_count_delete_clear_roundtrip() -> TestResult {
    if !configure_test_valkey() {
        return Ok(());
    }
    let temp_dir = TempDir::new()?;
    let storage = KnowledgeStorage::new(temp_dir.path().to_string_lossy().as_ref(), "knowledge");

    storage.init().await?;
    storage.clear().await?;

    let entry = KnowledgeEntry::new(
        "id-1".to_string(),
        "Rust Pattern".to_string(),
        "Use Result for error handling".to_string(),
        KnowledgeCategory::Pattern,
    )
    .with_tags(vec!["rust".to_string(), "error".to_string()]);
    storage.upsert(&entry).await?;
    assert_eq!(storage.count().await?, 1);

    let updated = KnowledgeEntry::new(
        "id-1".to_string(),
        "Rust Pattern Updated".to_string(),
        "Use anyhow for context-rich errors".to_string(),
        KnowledgeCategory::Pattern,
    )
    .with_tags(vec!["rust".to_string(), "anyhow".to_string()]);
    storage.upsert(&updated).await?;
    assert_eq!(storage.count().await?, 1);

    storage.delete("id-1").await?;
    assert_eq!(storage.count().await?, 0);

    storage.upsert(&entry).await?;
    storage.clear().await?;
    assert_eq!(storage.count().await?, 0);
    Ok(())
}

#[tokio::test]
async fn test_text_search_and_stats() -> TestResult {
    if !configure_test_valkey() {
        return Ok(());
    }
    let temp_dir = TempDir::new()?;
    let storage = KnowledgeStorage::new(temp_dir.path().to_string_lossy().as_ref(), "knowledge");
    storage.init().await?;
    storage.clear().await?;

    let e1 = KnowledgeEntry::new(
        "id-a".to_string(),
        "TypeScript Error Handling".to_string(),
        "Typed errors improve maintainability".to_string(),
        KnowledgeCategory::Pattern,
    )
    .with_tags(vec!["typescript".to_string(), "error".to_string()]);
    let e2 = KnowledgeEntry::new(
        "id-b".to_string(),
        "Workflow notes".to_string(),
        "This note describes deployment workflow".to_string(),
        KnowledgeCategory::Workflow,
    )
    .with_tags(vec!["deploy".to_string()]);

    storage.upsert(&e1).await?;
    storage.upsert(&e2).await?;

    let text_results = storage.search_text("typed error", 10).await?;
    assert_eq!(text_results.len(), 1);
    assert_eq!(text_results[0].id, "id-a");

    let vector_results = storage.search(&[0.1, 0.3, 0.2, 0.4], 2).await?;
    assert_eq!(vector_results.len(), 2);

    let stats = storage.stats().await?;
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.total_tags, 3, "stats={stats:?}");
    assert_eq!(stats.entries_by_category.get("patterns"), Some(&1));
    assert_eq!(stats.entries_by_category.get("workflows"), Some(&1));
    assert!(stats.last_updated.is_some());
    Ok(())
}

#[tokio::test]
async fn test_vector_search_prefers_semantically_closer_entry() -> TestResult {
    if !configure_test_valkey() {
        return Ok(());
    }
    let temp_dir = TempDir::new()?;
    let storage = KnowledgeStorage::new(temp_dir.path().to_string_lossy().as_ref(), "knowledge");
    storage.init().await?;
    storage.clear().await?;

    let e1 = KnowledgeEntry::new(
        "vec-1".to_string(),
        "Typed language benefits".to_string(),
        "Type systems catch compile-time errors and improve refactoring safety.".to_string(),
        KnowledgeCategory::Pattern,
    );
    let e2 = KnowledgeEntry::new(
        "vec-2".to_string(),
        "Deployment workflow".to_string(),
        "Release flow focuses on canary rollout and rollback strategy.".to_string(),
        KnowledgeCategory::Workflow,
    );

    storage.upsert(&e1).await?;
    storage.upsert(&e2).await?;

    let query =
        text_to_vector("Type systems catch compile-time errors and improve refactoring safety.");
    let hits = storage.search(&query, 1).await?;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "vec-1".to_string());
    Ok(())
}
