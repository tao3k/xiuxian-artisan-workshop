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

use crate::channels::traits::{Channel, RecipientCommandAdminUsersMutation};

use super::support::{
    MockChannel, build_agent, inbound, process_discord_message, start_job_manager,
};

#[tokio::test]
async fn process_discord_message_denies_unauthorized_control_command() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(false, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(agent, channel_dyn, inbound("/reset"), &job_manager, 10).await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("## Control Command Permission Denied"));
    assert!(sent[0].0.contains("`reason`: `admin_required`"));
    assert!(sent[0].0.contains("`command`: `/reset`"));
    Ok(())
}

#[tokio::test]
async fn process_discord_message_denies_unauthorized_slash_command() -> Result<()> {
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(true, ["session.memory"]));
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session memory"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("## Slash Command Permission Denied"));
    assert!(sent[0].0.contains("`reason`: `slash_permission_required`"));
    assert!(sent[0].0.contains("`command`: `/session memory`"));
    Ok(())
}

#[tokio::test]
async fn process_discord_message_allows_control_command_with_recipient_admin_override() -> Result<()>
{
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(false, std::iter::empty::<&str>()));
    channel.mutate_recipient_command_admin_users(
        "2001",
        RecipientCommandAdminUsersMutation::Set(vec!["1001".to_string()]),
    )?;
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(agent, channel_dyn, inbound("/reset"), &job_manager, 10).await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("Session context reset."));
    Ok(())
}

#[tokio::test]
async fn process_discord_message_allows_slash_command_with_recipient_admin_override() -> Result<()>
{
    let agent = build_agent().await?;
    let job_manager = start_job_manager(agent.clone());
    let channel = Arc::new(MockChannel::with_acl(false, ["session.memory"]));
    channel.mutate_recipient_command_admin_users(
        "2001",
        RecipientCommandAdminUsersMutation::Set(vec!["1001".to_string()]),
    )?;
    let channel_dyn: Arc<dyn Channel> = channel.clone();

    process_discord_message(
        agent,
        channel_dyn,
        inbound("/session memory"),
        &job_manager,
        10,
    )
    .await;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(!sent[0].0.contains("## Slash Command Permission Denied"));
    assert!(sent[0].0.contains("## Session Memory"));
    Ok(())
}
