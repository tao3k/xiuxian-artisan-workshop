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
use tokio::time::timeout;

use crate::channels::traits::Channel;

use super::support::{
    ForegroundInterruptController, MockChannel, build_agent, inbound,
    process_discord_message_with_interrupt, start_job_manager,
};

#[tokio::test]
async fn process_discord_message_preempts_same_session_foreground_generation() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();

    let msg = inbound("replace the previous answer with a concise summary");
    let session_id = format!("{}:{}", msg.channel, msg.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();

    process_discord_message_with_interrupt(
        agent,
        channel_dyn,
        msg,
        &job_manager,
        1,
        &interrupt_controller,
    )
    .await;

    timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "same-session plain message should preempt active generation"
    );

    interrupt_controller.end_generation(&session_id);
    Ok(())
}

#[tokio::test]
async fn process_discord_message_does_not_preempt_other_session_generation() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();

    let msg = inbound("continue");
    let other_session_id = "discord:other:session";
    let mut interrupt_rx = interrupt_controller.begin_generation(other_session_id);
    let initial_generation = *interrupt_rx.borrow();

    process_discord_message_with_interrupt(
        agent,
        channel_dyn,
        msg,
        &job_manager,
        1,
        &interrupt_controller,
    )
    .await;

    let changed = timeout(Duration::from_millis(120), interrupt_rx.changed()).await;
    assert!(
        changed.is_err(),
        "different-session message must not preempt active generation"
    );
    assert_eq!(
        *interrupt_rx.borrow(),
        initial_generation,
        "interrupt generation must remain unchanged for different sessions"
    );

    interrupt_controller.end_generation(other_session_id);
    Ok(())
}

#[tokio::test]
async fn process_discord_message_stop_command_interrupts_active_generation() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let interrupt_controller = ForegroundInterruptController::default();

    let stop_msg = inbound("/stop");
    let session_id = format!("{}:{}", stop_msg.channel, stop_msg.session_key);
    let mut interrupt_rx = interrupt_controller.begin_generation(&session_id);
    let initial_generation = *interrupt_rx.borrow();

    process_discord_message_with_interrupt(
        agent,
        channel_dyn,
        stop_msg,
        &job_manager,
        1,
        &interrupt_controller,
    )
    .await;

    timeout(Duration::from_millis(200), interrupt_rx.changed()).await??;
    assert!(
        *interrupt_rx.borrow() > initial_generation,
        "stop command should preempt the active generation"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("Stop signal sent."));

    interrupt_controller.end_generation(&session_id);
    Ok(())
}
