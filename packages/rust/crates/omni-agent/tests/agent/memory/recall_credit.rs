#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use anyhow::Result;
use omni_memory::{Episode, EpisodeStore, StoreConfig};

use super::{apply_recall_credit, select_recall_credit_candidates};
use crate::agent::memory_recall_feedback::RecallOutcome;

fn new_store() -> EpisodeStore {
    let tmp = tempfile::tempdir().expect("tempdir");
    EpisodeStore::new(StoreConfig {
        path: tmp.path().join("memory").to_string_lossy().to_string(),
        embedding_dim: 8,
        table_name: "agent_recall_credit".to_string(),
    })
}

fn episode(id: &str) -> Episode {
    Episode::new(
        id.to_string(),
        format!("intent-{id}"),
        vec![0.1; 8],
        format!("experience-{id}"),
        "completed".to_string(),
    )
}

#[test]
fn select_recall_credit_candidates_keeps_rank_order_and_limit() -> Result<()> {
    let store = new_store();
    store.store(episode("ep-1"))?;
    store.store(episode("ep-2"))?;
    store.store(episode("ep-3"))?;

    let recalled = vec![
        (store.get("ep-1").expect("ep-1"), 0.91),
        (store.get("ep-2").expect("ep-2"), 0.72),
        (store.get("ep-3").expect("ep-3"), 0.61),
    ];

    let selected = select_recall_credit_candidates(&recalled, 2);
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].episode_id, "ep-1");
    assert_eq!(selected[1].episode_id, "ep-2");
    Ok(())
}

#[test]
fn apply_recall_credit_success_increases_q_and_tracks_success() -> Result<()> {
    let store = new_store();
    store.store(episode("ep-1"))?;
    store.update_q("ep-1", 0.2);
    let candidates = vec![super::RecalledEpisodeCandidate {
        episode_id: "ep-1".to_string(),
        score: 0.9,
    }];

    let updates = apply_recall_credit(&store, &candidates, RecallOutcome::Success);
    assert_eq!(updates.len(), 1);
    assert!(updates[0].updated_q > updates[0].previous_q);

    let ep = store.get("ep-1").expect("episode should exist");
    assert_eq!(ep.success_count, 1);
    assert_eq!(ep.failure_count, 0);
    Ok(())
}

#[test]
fn apply_recall_credit_failure_decreases_q_and_tracks_failure() -> Result<()> {
    let store = new_store();
    store.store(episode("ep-1"))?;
    store.update_q("ep-1", 0.9);
    let candidates = vec![super::RecalledEpisodeCandidate {
        episode_id: "ep-1".to_string(),
        score: 0.8,
    }];

    let updates = apply_recall_credit(&store, &candidates, RecallOutcome::Failure);
    assert_eq!(updates.len(), 1);
    assert!(updates[0].updated_q < updates[0].previous_q);

    let ep = store.get("ep-1").expect("episode should exist");
    assert_eq!(ep.success_count, 0);
    assert_eq!(ep.failure_count, 1);
    Ok(())
}
