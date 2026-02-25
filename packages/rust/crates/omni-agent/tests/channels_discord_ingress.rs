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

use std::time::Duration;

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use omni_agent::{
    DiscordSessionPartition, build_discord_ingress_app,
    build_discord_ingress_app_with_partition_and_control_command_policy,
};
use tokio::sync::mpsc;
use tower::util::ServiceExt;

const DISCORD_INGRESS_SECRET_HEADER: &str = "x-omni-discord-ingress-token";

fn sample_event(
    message_id: &str,
    user_id: &str,
    username: &str,
    channel_id: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": message_id,
        "content": "hello",
        "channel_id": channel_id,
        "guild_id": "3001",
        "author": {
            "id": user_id,
            "username": username
        }
    })
}

fn sample_slash_interaction_event(
    interaction_id: &str,
    user_id: &str,
    username: &str,
    channel_id: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": interaction_id,
        "application_id": "5001",
        "type": 2,
        "data": {
            "id": "6001",
            "name": "session",
            "type": 1,
            "options": [
                {
                    "name": "memory",
                    "type": 1,
                    "options": [
                        {
                            "name": "format",
                            "type": 3,
                            "value": "json"
                        }
                    ]
                }
            ]
        },
        "channel_id": channel_id,
        "guild_id": "3001",
        "token": "interaction-token",
        "version": 1,
        "locale": "en-US",
        "guild_locale": "en-US",
        "entitlements": [],
        "attachment_size_limit": 8388608,
        "user": {
            "id": user_id,
            "username": username
        }
    })
}

async fn post_event(
    app: Router,
    path: &str,
    payload: serde_json::Value,
    secret_token: Option<&str>,
) -> Result<StatusCode> {
    let mut request_builder = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(secret) = secret_token {
        request_builder = request_builder.header(DISCORD_INGRESS_SECRET_HEADER, secret);
    }
    let request = request_builder.body(Body::from(payload.to_string()))?;
    let response = app.oneshot(request).await?;
    Ok(response.status())
}

#[tokio::test]
async fn discord_ingress_rejects_invalid_secret_without_enqueue() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let ingress = build_discord_ingress_app(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        "/discord/ingress",
        Some("expected-secret".to_string()),
        tx,
    )?;

    let status = post_event(
        ingress.app.clone(),
        &ingress.path,
        sample_event("1", "1001", "alice", "2001"),
        Some("wrong-secret"),
    )
    .await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(
        tokio::time::timeout(Duration::from_millis(120), rx.recv())
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn discord_ingress_enqueues_authorized_event() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let ingress = build_discord_ingress_app(
        "fake-token".to_string(),
        vec!["alice".to_string()],
        vec![],
        "/discord/ingress",
        None,
        tx,
    )?;

    let status = post_event(
        ingress.app.clone(),
        &ingress.path,
        sample_event("1", "1001", "alice", "2001"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let message = tokio::time::timeout(Duration::from_millis(250), rx.recv())
        .await?
        .expect("message should be queued");
    assert_eq!(message.channel, "discord");
    assert_eq!(message.recipient, "2001");
    assert_eq!(message.session_key, "3001:2001:1001");
    Ok(())
}

#[tokio::test]
async fn discord_ingress_ignores_unauthorized_event() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let ingress = build_discord_ingress_app(
        "fake-token".to_string(),
        vec!["owner".to_string()],
        vec![],
        "/discord/ingress",
        None,
        tx,
    )?;

    let status = post_event(
        ingress.app.clone(),
        &ingress.path,
        sample_event("1", "1001", "alice", "2001"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(
        tokio::time::timeout(Duration::from_millis(120), rx.recv())
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn discord_ingress_partition_channel_only_shares_session() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let ingress = build_discord_ingress_app_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        omni_agent::DiscordControlCommandPolicy::new(vec!["*".to_string()], None, Vec::new()),
        "/discord/ingress",
        None,
        DiscordSessionPartition::ChannelOnly,
        tx,
    )?;

    assert_eq!(
        post_event(
            ingress.app.clone(),
            &ingress.path,
            sample_event("1", "1001", "alice", "2001"),
            None,
        )
        .await?,
        StatusCode::OK
    );
    assert_eq!(
        post_event(
            ingress.app.clone(),
            &ingress.path,
            sample_event("2", "1002", "bob", "2001"),
            None,
        )
        .await?,
        StatusCode::OK
    );

    let first = tokio::time::timeout(Duration::from_millis(250), rx.recv())
        .await?
        .expect("first message");
    let second = tokio::time::timeout(Duration::from_millis(250), rx.recv())
        .await?
        .expect("second message");
    assert_eq!(first.session_key, "3001:2001");
    assert_eq!(first.session_key, second.session_key);
    Ok(())
}

#[tokio::test]
async fn discord_ingress_enqueues_managed_slash_interaction_event() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let ingress = build_discord_ingress_app(
        "fake-token".to_string(),
        vec!["alice".to_string()],
        vec![],
        "/discord/ingress",
        None,
        tx,
    )?;

    let status = post_event(
        ingress.app.clone(),
        &ingress.path,
        sample_slash_interaction_event("9", "1001", "alice", "2001"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let message = tokio::time::timeout(Duration::from_millis(250), rx.recv())
        .await?
        .expect("interaction should be queued");
    assert_eq!(message.channel, "discord");
    assert_eq!(message.content, "/session memory json");
    assert_eq!(message.recipient, "2001");
    assert_eq!(message.session_key, "3001:2001:1001");
    Ok(())
}
