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

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use omni_agent::{Agent, AgentConfig};

fn unique_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

async fn build_agent() -> Result<Agent> {
    let mut config = AgentConfig::default();
    config.inference_url = "http://127.0.0.1:4000/v1/chat/completions".to_string();
    config.memory = None;
    config.window_max_turns = None;
    config.consolidation_threshold_turns = None;
    Agent::from_config(config).await
}

#[tokio::test]
async fn upsert_and_inspect_system_prompt_injection_roundtrip() -> Result<()> {
    let agent = build_agent().await?;
    let session_id = unique_id("system-prompt-injection-roundtrip");
    let xml = r#"
<system_prompt_injection>
  <qa>
    <q>What backend should we use?</q>
    <a>Use valkey for session/memory state.</a>
  </qa>
  <qa>
    <q>What fallback should be avoided?</q>
    <a>Do not use local json fallback in production.</a>
  </qa>
</system_prompt_injection>
"#;

    let snapshot = agent
        .upsert_session_system_prompt_injection_xml(&session_id, xml)
        .await?;
    assert_eq!(snapshot.qa_count, 2);
    assert!(snapshot.xml.contains("<system_prompt_injection>"));

    let loaded = agent
        .inspect_session_system_prompt_injection(&session_id)
        .await
        .expect("snapshot should exist");
    assert_eq!(loaded.qa_count, 2);
    assert!(loaded.xml.contains("<q>What backend should we use?</q>"));
    Ok(())
}

#[tokio::test]
async fn clear_system_prompt_injection_is_idempotent() -> Result<()> {
    let agent = build_agent().await?;
    let session_id = unique_id("system-prompt-injection-clear");
    let xml = "<qa><q>q</q><a>a</a></qa>";

    let _ = agent
        .upsert_session_system_prompt_injection_xml(&session_id, xml)
        .await?;
    assert!(
        agent
            .clear_session_system_prompt_injection(&session_id)
            .await?
    );
    assert!(
        !agent
            .clear_session_system_prompt_injection(&session_id)
            .await?
    );
    assert!(
        agent
            .inspect_session_system_prompt_injection(&session_id)
            .await
            .is_none()
    );
    Ok(())
}

#[tokio::test]
async fn upsert_system_prompt_injection_rejects_invalid_xml() -> Result<()> {
    let agent = build_agent().await?;
    let session_id = unique_id("system-prompt-injection-invalid");
    let invalid = "<qa><q>question only</q></qa>";

    let error = agent
        .upsert_session_system_prompt_injection_xml(&session_id, invalid)
        .await
        .expect_err("invalid payload should fail");
    assert!(
        error
            .to_string()
            .contains("invalid system prompt injection xml payload")
    );
    Ok(())
}
