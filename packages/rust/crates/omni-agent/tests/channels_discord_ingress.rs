//! Discord ingress endpoint tests for auth, routing, and partition behavior.

use std::time::Duration;

use anyhow::Result;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use omni_agent::{
    DiscordIngressBuildRequest, DiscordSessionPartition, build_discord_ingress_app,
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
        "attachment_size_limit": 8_388_608,
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
    let message = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(message) = message else {
        panic!("message should be queued");
    };
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
        DiscordIngressBuildRequest {
            bot_token: "fake-token".to_string(),
            allowed_users: vec!["*".to_string()],
            allowed_guilds: vec![],
            control_command_policy: omni_agent::DiscordControlCommandPolicy::new(
                vec!["*".to_string()],
                None,
                Vec::new(),
            ),
            ingress_path: "/discord/ingress".to_string(),
            secret_token: None,
            session_partition: DiscordSessionPartition::ChannelOnly,
            tx,
        },
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

    let first = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(first) = first else {
        panic!("first message");
    };
    let second = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(second) = second else {
        panic!("second message");
    };
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
    let message = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(message) = message else {
        panic!("interaction should be queued");
    };
    assert_eq!(message.channel, "discord");
    assert_eq!(message.content, "/session memory json");
    assert_eq!(message.recipient, "2001");
    assert_eq!(message.session_key, "3001:2001:1001");
    Ok(())
}
