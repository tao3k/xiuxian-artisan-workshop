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
use omni_agent::{BoundedSessionStore, SessionSummarySegment};

fn sample_segment(
    summary: &str,
    turns: usize,
    tools: u32,
    created_at_ms: u64,
) -> SessionSummarySegment {
    SessionSummarySegment::new(summary.to_string(), turns, tools, created_at_ms)
}

#[tokio::test]
async fn bounded_store_summary_is_bounded_and_trimmed() -> Result<()> {
    let store = BoundedSessionStore::new_with_limits(16, 2, 20)?;
    let session_id = "s-summary-bounded";

    store
        .append_summary_segment(
            session_id,
            sample_segment(
                "first summary should be dropped after third insert",
                2,
                1,
                100,
            ),
        )
        .await?;
    store
        .append_summary_segment(
            session_id,
            sample_segment("second summary is kept", 3, 0, 101),
        )
        .await?;
    store
        .append_summary_segment(
            session_id,
            sample_segment("third summary is kept too", 1, 2, 102),
        )
        .await?;

    let summaries = store.get_recent_summary_segments(session_id, 10).await?;
    assert_eq!(summaries.len(), 2);
    assert!(summaries[0].summary.starts_with("second"));
    assert!(summaries[1].summary.starts_with("third"));
    assert!(summaries[0].summary.chars().count() <= 20);
    assert!(summaries[1].summary.chars().count() <= 20);
    Ok(())
}

#[tokio::test]
async fn clear_session_removes_summary_segments() -> Result<()> {
    let store = BoundedSessionStore::new_with_limits(8, 4, 128)?;
    let session_id = "s-summary-clear";

    store.append_turn(session_id, "u1", "a1", 1).await?;
    store
        .append_summary_segment(session_id, sample_segment("done", 1, 1, 200))
        .await?;

    assert!(store.get_stats(session_id).await?.is_some());
    assert_eq!(
        store
            .get_recent_summary_segments(session_id, 10)
            .await?
            .len(),
        1
    );

    store.clear(session_id).await?;

    assert!(store.get_stats(session_id).await?.is_none());
    assert!(
        store
            .get_recent_summary_segments(session_id, 10)
            .await?
            .is_empty()
    );
    Ok(())
}
