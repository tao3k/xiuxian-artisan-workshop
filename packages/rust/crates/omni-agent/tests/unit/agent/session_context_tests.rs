use anyhow::Result;

use crate::agent::Agent;
use crate::config::AgentConfig;

use super::now_unix_ms;

const SESSION_RESET_NOTICE_NAME: &str = "session_reset_notice";
const IDLE_RESET_NOTICE_TEXT: &str = "Previous session expired due to inactivity.";

async fn build_agent(window_max_turns: Option<usize>) -> Result<Agent> {
    let config = AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        window_max_turns,
        memory: None,
        consolidation_threshold_turns: None,
        ..AgentConfig::default()
    };
    Agent::from_config(config).await
}

#[tokio::test]
async fn enforce_session_reset_policy_resets_stale_unbounded_session_and_injects_notice()
-> Result<()> {
    let mut agent = build_agent(None).await?;
    agent.session_reset_idle_timeout_ms = Some(1);
    let session_id = "session-reset-unbounded";

    agent
        .append_turn_for_session(session_id, "u1", "a1")
        .await?;
    let stale_ms = now_unix_ms().saturating_sub(10);
    agent
        .session_last_activity_unix_ms
        .write()
        .await
        .insert(session_id.to_string(), stale_ms);

    agent.enforce_session_reset_policy(session_id).await?;

    let messages = agent.session.get(session_id).await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].name.as_deref(), Some(SESSION_RESET_NOTICE_NAME));
    assert_eq!(messages[0].content.as_deref(), Some(IDLE_RESET_NOTICE_TEXT));

    let backup = agent.peek_context_window_backup(session_id).await?;
    assert!(
        backup.is_some(),
        "stale reset should preserve backup snapshot"
    );

    Ok(())
}

#[tokio::test]
async fn enforce_session_reset_policy_resets_stale_bounded_session_and_injects_summary_notice()
-> Result<()> {
    let mut agent = build_agent(Some(8)).await?;
    agent.session_reset_idle_timeout_ms = Some(1);
    let session_id = "session-reset-bounded";

    agent
        .append_turn_for_session(session_id, "u1", "a1")
        .await?;
    agent
        .append_turn_for_session(session_id, "u2", "a2")
        .await?;
    let stale_ms = now_unix_ms().saturating_sub(10);
    agent
        .session_last_activity_unix_ms
        .write()
        .await
        .insert(session_id.to_string(), stale_ms);

    agent.enforce_session_reset_policy(session_id).await?;

    let Some(ref bounded) = agent.bounded_session else {
        panic!("bounded session store expected");
    };
    let recent_messages = bounded.get_recent_messages(session_id, 16).await?;
    assert!(
        recent_messages.is_empty(),
        "stale reset should clear active window"
    );

    let summary_segments = bounded.get_recent_summary_segments(session_id, 8).await?;
    assert_eq!(summary_segments.len(), 1);
    assert_eq!(summary_segments[0].summary, IDLE_RESET_NOTICE_TEXT);

    let backup = agent.peek_context_window_backup(session_id).await?;
    assert!(
        backup.is_some(),
        "stale reset should preserve backup snapshot"
    );

    Ok(())
}
