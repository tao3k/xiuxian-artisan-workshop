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
use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use crate::channels::traits::{Channel, ChannelMessage};

use super::{MockChannel, build_agent, build_job_manager, handle_inbound_message, inbound};

#[derive(Default)]
struct DenySlashMockChannel {
    sent: Mutex<Vec<(String, String)>>,
}

impl DenySlashMockChannel {
    async fn sent_messages(&self) -> Vec<(String, String)> {
        self.sent.lock().await.clone()
    }
}

#[async_trait]
impl Channel for DenySlashMockChannel {
    fn name(&self) -> &str {
        "deny-slash-mock"
    }

    fn is_authorized_for_slash_command(&self, _identity: &str, _command_scope: &str) -> bool {
        false
    }

    async fn send(&self, message: &str, recipient: &str) -> Result<()> {
        self.sent
            .lock()
            .await
            .push((message.to_string(), recipient.to_string()));
        Ok(())
    }

    async fn listen(&self, _tx: mpsc::Sender<ChannelMessage>) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn runtime_handle_inbound_help_replies_with_command_guide() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    assert!(
        handle_inbound_message(
            inbound("/help"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(
        foreground_rx.try_recv().is_err(),
        "help command should not forward to foreground queue"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("## Bot Slash Help"));
    assert!(sent[0].0.contains("- `/session memory [json]`"));
    assert!(
        sent[0]
            .0
            .contains("- `/session admin [list|set|add|remove|clear] [json]`")
    );
    assert!(sent[0].0.contains("- `/bg <prompt>`"));
    assert!(sent[0].0.contains("- `/help json`"));
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_help_json_replies_with_machine_readable_catalog() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(MockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    assert!(
        handle_inbound_message(
            inbound("/slash help json"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(
        foreground_rx.try_recv().is_err(),
        "help json command should not forward to foreground queue"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&sent[0].0)?;
    assert_eq!(payload["kind"], "slash_help");
    assert!(payload["commands"]["general"].is_array());
    assert!(payload["commands"]["session"].is_array());
    assert!(
        payload["commands"]["session"]
            .as_array()
            .is_some_and(|commands| commands.iter().any(|entry| {
                entry
                    .get("usage")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|usage| usage.starts_with("/session admin "))
            }))
    );
    assert!(payload["commands"]["background"].is_array());
    Ok(())
}

#[tokio::test]
async fn runtime_handle_inbound_help_is_not_blocked_by_slash_acl() -> Result<()> {
    let agent = build_agent().await?;
    let channel = Arc::new(DenySlashMockChannel::default());
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let job_manager = build_job_manager(agent.clone());
    let (foreground_tx, mut foreground_rx) = mpsc::channel::<ChannelMessage>(8);

    assert!(
        handle_inbound_message(
            inbound("/help"),
            &channel_dyn,
            &foreground_tx,
            &job_manager,
            &agent,
        )
        .await
    );
    assert!(
        foreground_rx.try_recv().is_err(),
        "help command should not forward to foreground queue"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(sent[0].0.contains("## Bot Slash Help"));
    assert!(!sent[0].0.contains("slash_permission_required"));
    Ok(())
}
