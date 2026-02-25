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
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::channels::telegram::runtime::dispatch::ForegroundInterruptController;
use crate::channels::traits::Channel;

use super::{
    MockChannel, build_agent, build_job_manager, handle_inbound_message_with_interrupt, inbound,
};

#[tokio::test]
async fn runtime_handle_inbound_plain_message_preempts_active_session_generation() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);
    let interrupt_controller = ForegroundInterruptController::default();

    let message = inbound("replace the prior answer with a concise summary");
    let expected_content = message.content.clone();
    let session_id = format!("{}:{}", message.channel, message.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();

    let handled = handle_inbound_message_with_interrupt(
        message,
        &channel_dyn,
        &foreground_tx,
        &interrupt_controller,
        &job_manager,
        &agent,
    )
    .await;
    assert!(handled, "plain message should be accepted by runtime");

    let queued = foreground_rx
        .recv()
        .await
        .expect("plain message should enter foreground queue");
    assert_eq!(queued.content, expected_content);

    tokio::time::timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "new plain message should preempt active generation in same session"
    );

    assert!(
        channel.sent_messages().await.is_empty(),
        "preemption should not emit immediate control reply"
    );

    interrupt_controller.end_generation(&session_id);
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_plain_message_does_not_preempt_other_session() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel(8);
    let interrupt_controller = ForegroundInterruptController::default();

    let message = inbound("continue");
    let other_session_id = "telegram:-200:999";
    let mut interrupt_rx = interrupt_controller.begin_generation(other_session_id);
    let initial_generation = *interrupt_rx.borrow();

    let handled = handle_inbound_message_with_interrupt(
        message,
        &channel_dyn,
        &foreground_tx,
        &interrupt_controller,
        &job_manager,
        &agent,
    )
    .await;
    assert!(handled, "plain message should be accepted by runtime");
    assert!(
        foreground_rx.recv().await.is_some(),
        "plain message should enter foreground queue"
    );

    let changed = tokio::time::timeout(Duration::from_millis(120), interrupt_rx.changed()).await;
    assert!(
        changed.is_err(),
        "message from different session must not preempt active generation"
    );
    assert_eq!(
        *interrupt_rx.borrow(),
        initial_generation,
        "interrupt generation should stay unchanged for other sessions"
    );

    assert!(
        channel.sent_messages().await.is_empty(),
        "cross-session no-op should not emit control reply"
    );

    interrupt_controller.end_generation(other_session_id);
    Ok(())
}
