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

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::channels::traits::{Channel, ChannelMessage};

use super::{MockChannel, build_agent, build_job_manager, handle_inbound_message, inbound};

#[tokio::test]
async fn runtime_handle_inbound_session_reset_resume_flow() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);
    let session_id = "telegram:-200:888";

    agent
        .append_turn_for_session(session_id, "u1", "a1")
        .await?;
    agent
        .append_turn_for_session(session_id, "u2", "a2")
        .await?;

    let reset_handled = handle_inbound_message(
        inbound("/reset"),
        &channel_dyn,
        &foreground_tx,
        &job_manager,
        &agent,
    )
    .await;
    assert!(reset_handled, "reset command should be handled by runtime");
    assert!(
        foreground_rx.try_recv().is_err(),
        "session commands should not forward to foreground queue"
    );

    let status_handled = handle_inbound_message(
        inbound("/resume status"),
        &channel_dyn,
        &foreground_tx,
        &job_manager,
        &agent,
    )
    .await;
    assert!(status_handled, "resume status should be handled by runtime");

    let resume_handled = handle_inbound_message(
        inbound("/resume"),
        &channel_dyn,
        &foreground_tx,
        &job_manager,
        &agent,
    )
    .await;
    assert!(resume_handled, "resume should be handled by runtime");

    let status_after_resume_handled = handle_inbound_message(
        inbound("/resume status"),
        &channel_dyn,
        &foreground_tx,
        &job_manager,
        &agent,
    )
    .await;
    assert!(
        status_after_resume_handled,
        "resume status should still be handled after restore"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 4);
    assert!(sent[0].0.contains("Session context reset."));
    assert!(sent[0].0.contains("messages_cleared=4"));
    assert!(sent[1].0.contains("Saved session context snapshot:"));
    assert!(sent[1].0.contains("messages=4"));
    assert!(sent[1].0.contains("saved_age_secs="));
    assert!(sent[2].0.contains("Session context restored."));
    assert!(sent[2].0.contains("messages_restored=4"));
    assert!(
        sent[3]
            .0
            .contains("No saved session context snapshot found.")
    );
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_resume_status_without_snapshot() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    let handled = handle_inbound_message(
        inbound("/resume status"),
        &channel_dyn,
        &foreground_tx,
        &job_manager,
        &agent,
    )
    .await;

    assert!(handled);
    assert!(
        foreground_rx.try_recv().is_err(),
        "resume status should not forward to foreground queue"
    );
    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0]
            .0
            .contains("No saved session context snapshot found.")
    );
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_resume_drop_clears_snapshot() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);
    let session_id = "telegram:-200:888";

    agent
        .append_turn_for_session(session_id, "u1", "a1")
        .await?;
    agent
        .append_turn_for_session(session_id, "u2", "a2")
        .await?;

    assert!(
        handle_inbound_message(
            inbound("/reset"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(
        handle_inbound_message(
            inbound("/resume drop"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(
        handle_inbound_message(
            inbound("/resume status"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );

    assert!(
        foreground_rx.try_recv().is_err(),
        "session commands should not forward to foreground queue"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 3);
    assert!(sent[0].0.contains("Session context reset."));
    assert!(
        sent[1]
            .0
            .contains("Saved session context snapshot dropped.")
    );
    assert!(
        sent[2]
            .0
            .contains("No saved session context snapshot found.")
    );
    Ok(())
}
