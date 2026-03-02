//! Feedback tracking integration tests for `omni-memory`.

use anyhow::Result;
use omni_memory::{Episode, EpisodeStore, RecallFeedbackOutcome, StoreConfig};

const FLOAT_EPSILON: f32 = 1.0e-6;

fn assert_f32_eq(left: f32, right: f32) {
    assert!((left - right).abs() <= FLOAT_EPSILON);
}

fn new_store() -> Result<EpisodeStore> {
    let tmp = tempfile::tempdir()?;
    Ok(EpisodeStore::new(StoreConfig {
        path: tmp.path().join("memory").to_string_lossy().into_owned(),
        embedding_dim: 8,
        table_name: "feedback_tracking".to_string(),
    }))
}

fn episode(id: &str) -> Episode {
    Episode::new(
        id.to_string(),
        "intent".to_string(),
        vec![0.1; 8],
        "experience".to_string(),
        "completed".to_string(),
    )
}

#[test]
fn record_feedback_updates_success_and_failure_counts() -> Result<()> {
    let store = new_store()?;
    store.store(episode("ep-1"))?;

    assert!(store.record_feedback("ep-1", true));
    assert!(store.record_feedback("ep-1", false));

    let ep = store
        .get("ep-1")
        .ok_or_else(|| anyhow::anyhow!("episode should exist"))?;
    assert_eq!(ep.success_count, 1);
    assert_eq!(ep.failure_count, 1);
    Ok(())
}

#[test]
fn record_feedback_returns_false_for_missing_episode() -> Result<()> {
    let store = new_store()?;
    assert!(!store.record_feedback("missing", true));
    Ok(())
}

#[test]
fn scope_feedback_bias_update_and_clear_roundtrip() -> Result<()> {
    let store = new_store()?;
    let scope = "session-feedback-1";

    assert_f32_eq(store.recall_feedback_bias_for_scope(scope), 0.0);

    let (previous, updated) =
        store.apply_recall_feedback_for_scope(scope, RecallFeedbackOutcome::Failure);
    assert_f32_eq(previous, 0.0);
    assert!(updated < 0.0);
    assert_f32_eq(store.recall_feedback_bias_for_scope(scope), updated);

    let before_success = store.recall_feedback_bias_for_scope(scope);
    let (previous, updated) =
        store.apply_recall_feedback_for_scope(scope, RecallFeedbackOutcome::Success);
    assert_f32_eq(previous, before_success);
    assert!(updated > previous);

    assert!(store.clear_recall_feedback_bias_for_scope(scope));
    assert_f32_eq(store.recall_feedback_bias_for_scope(scope), 0.0);
    Ok(())
}
