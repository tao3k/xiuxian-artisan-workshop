use crate::agent::Agent;
use crate::channels::managed_runtime::turn::{
    ForegroundTurnOutcome, ForegroundTurnRequest, build_session_id,
    run_foreground_turn_with_interrupt,
};
use crate::channels::traits::{Channel, ChannelMessage};
use std::sync::Arc;
use tokio::sync::watch;

use super::preview::{log_preview, sanitize_reply_for_send};

const TASK_ID_PREFIX: &str = "task:";

pub(super) async fn process_foreground_message(
    agent: Arc<Agent>,
    channel: Arc<dyn Channel>,
    msg: ChannelMessage,
    turn_timeout_secs: u64,
    interrupt_rx: watch::Receiver<u64>,
) {
    let session_id = build_session_id(&msg.channel, &msg.session_key);
    tracing::info!(
        r#"← User: "{preview}""#,
        preview = log_preview(&msg.content)
    );

    if let Err(error) = channel.start_typing(&msg.recipient).await {
        tracing::debug!("Failed to start typing: {error}");
    }

    let interrupt_generation = *interrupt_rx.borrow();
    let result = run_foreground_turn_with_interrupt(ForegroundTurnRequest {
        agent: Arc::clone(&agent),
        session_id: session_id.clone(),
        content: msg.content.clone(),
        timeout_secs: turn_timeout_secs,
        timeout_reply: format!(
            "Request timed out after {turn_timeout_secs}s. Use `/bg <prompt>` for long-running tasks."
        ),
        interrupt_rx,
        interrupt_generation,
        interrupted_reply: "Request interrupted by a newer instruction.".to_string(),
    })
    .await;

    if let Err(error) = channel.stop_typing(&msg.recipient).await {
        tracing::debug!("Failed to stop typing: {error}");
    }

    let raw_reply = match result {
        ForegroundTurnOutcome::Succeeded(output) => output,
        ForegroundTurnOutcome::Failed {
            reply,
            error_chain,
            error_kind,
        } => {
            tracing::error!(
                event = "telegram.foreground.turn.failed",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                error_kind,
                error = %error_chain,
                "foreground turn failed"
            );
            reply
        }
        ForegroundTurnOutcome::TimedOut { reply } => {
            tracing::warn!(
                event = "telegram.foreground.turn.timeout",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                timeout_secs = turn_timeout_secs,
                "foreground turn timed out"
            );
            reply
        }
        ForegroundTurnOutcome::Interrupted { reply } => {
            tracing::warn!(
                event = "telegram.foreground.turn.interrupted",
                session_key = %msg.session_key,
                channel = %msg.channel,
                recipient = %msg.recipient,
                sender = %msg.sender,
                "foreground turn interrupted by control signal"
            );
            reply
        }
    };
    let sanitized_reply = sanitize_reply_for_send(&raw_reply);
    let reply = normalize_task_id_only_reply(agent.as_ref(), sanitized_reply);

    match channel.send(&reply, &msg.recipient).await {
        Ok(()) => tracing::info!(r#"→ Bot: "{preview}""#, preview = log_preview(&reply)),
        Err(error) => tracing::error!("Failed to send foreground reply: {error}"),
    }

    // NATIVE INTEGRATION: Trigger library-level sync instead of calling binary
    if let Some(ref heyi) = agent.get_heyi() {
        match heyi.sync_from_disk() {
            Ok(summary) => {
                tracing::debug!(
                    event = "telegram.zhixing.sync.completed",
                    journal_documents = summary.journal_documents,
                    agenda_documents = summary.agenda_documents,
                    task_entities = summary.task_entities,
                    entities_added = summary.entities_added,
                    relations_linked = summary.relations_linked,
                    "Zhixing-Heyi library-level sync succeeded"
                );
            }
            Err(error) => {
                tracing::warn!(
                    event = "telegram.zhixing.sync.failed",
                    error = %error,
                    "Zhixing-Heyi library-level sync failed"
                );
            }
        }
    }
}

pub(super) fn extract_task_id_only_reply(reply: &str) -> Option<&str> {
    let trimmed = reply.trim();
    if !trimmed.starts_with(TASK_ID_PREFIX) || trimmed.contains(char::is_whitespace) {
        return None;
    }
    let suffix = &trimmed[TASK_ID_PREFIX.len()..];
    if suffix.len() < 8 {
        return None;
    }
    if suffix.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-') {
        Some(trimmed)
    } else {
        None
    }
}

fn normalize_task_id_only_reply(agent: &Agent, reply: String) -> String {
    let Some(task_id) = extract_task_id_only_reply(&reply) else {
        return reply;
    };
    let Some(heyi) = agent.get_heyi() else {
        return reply;
    };
    match heyi.render_task_add_response_from_id(task_id) {
        Ok(rendered) => {
            tracing::info!(
                event = "telegram.foreground.reply.normalized",
                task_id,
                "normalized bare task id reply to structured task confirmation"
            );
            rendered
        }
        Err(error) => {
            tracing::warn!(
                event = "telegram.foreground.reply.normalize_failed",
                task_id,
                error = %error,
                "failed to normalize bare task id reply"
            );
            reply
        }
    }
}
