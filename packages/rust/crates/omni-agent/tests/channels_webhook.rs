//! Telegram webhook endpoint tests for auth, dedup, and command forwarding.

use std::time::Duration;

use anyhow::{Result, anyhow};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use omni_agent::{
    DEFAULT_REDIS_KEY_PREFIX, TelegramSessionPartition, TelegramWebhookPartitionBuildRequest,
    WebhookDedupBackend, WebhookDedupConfig, build_telegram_webhook_app,
    build_telegram_webhook_app_with_partition,
};
use tokio::sync::mpsc;
use tower::util::ServiceExt;

const TELEGRAM_WEBHOOK_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

fn sample_update(update_id: i64) -> serde_json::Value {
    serde_json::json!({
        "update_id": update_id,
        "message": {
            "message_id": 77,
            "text": "hello",
            "chat": {"id": -200_123},
            "from": {"id": 888, "username": "alice"}
        }
    })
}

fn sample_update_with_identity(
    update_id: i64,
    message_id: i64,
    chat_id: i64,
    user_id: i64,
    message_thread_id: Option<i64>,
) -> serde_json::Value {
    let mut update = serde_json::json!({
        "update_id": update_id,
        "message": {
            "message_id": message_id,
            "text": "hello",
            "chat": {"id": chat_id},
            "from": {"id": user_id, "username": format!("u{user_id}")}
        }
    });
    if let Some(thread_id) = message_thread_id {
        update["message"]["message_thread_id"] = serde_json::json!(thread_id);
    }
    update
}

async fn post_update(
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
        request_builder = request_builder.header(TELEGRAM_WEBHOOK_SECRET_HEADER, secret);
    }

    let request = request_builder.body(Body::from(payload.to_string()))?;
    let response = app.oneshot(request).await?;
    Ok(response.status())
}

#[tokio::test]
async fn webhook_rejects_invalid_secret_without_enqueue() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook = build_telegram_webhook_app(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        "/telegram/webhook",
        Some("expected-secret".to_string()),
        WebhookDedupConfig {
            backend: WebhookDedupBackend::Memory,
            ttl_secs: 600,
        },
        tx,
    )?;

    let status = post_update(
        webhook.app.clone(),
        &webhook.path,
        sample_update(10001),
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
async fn webhook_dedups_duplicate_update_id() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook = build_telegram_webhook_app(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        "/telegram/webhook",
        None,
        WebhookDedupConfig {
            backend: WebhookDedupBackend::Memory,
            ttl_secs: 600,
        },
        tx,
    )?;

    let first = post_update(
        webhook.app.clone(),
        &webhook.path,
        sample_update(20001),
        None,
    )
    .await?;
    assert_eq!(first, StatusCode::OK);
    let first_msg = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await?;
    assert!(first_msg.is_some());

    let second = post_update(
        webhook.app.clone(),
        &webhook.path,
        sample_update(20001),
        None,
    )
    .await?;
    assert_eq!(second, StatusCode::OK);
    assert!(
        tokio::time::timeout(Duration::from_millis(120), rx.recv())
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn webhook_fail_open_when_valkey_is_unavailable() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook = build_telegram_webhook_app(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        "/telegram/webhook",
        None,
        WebhookDedupConfig {
            backend: WebhookDedupBackend::Redis {
                url: "redis://127.0.0.1:1/0".to_string(),
                key_prefix: DEFAULT_REDIS_KEY_PREFIX.to_string(),
            },
            ttl_secs: 600,
        },
        tx,
    )?;

    let first = post_update(
        webhook.app.clone(),
        &webhook.path,
        sample_update(30001),
        None,
    )
    .await?;
    let second = post_update(
        webhook.app.clone(),
        &webhook.path,
        sample_update(30001),
        None,
    )
    .await?;
    assert_eq!(first, StatusCode::OK);
    assert_eq!(second, StatusCode::OK);

    let first_msg = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let second_msg = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    assert!(first_msg.is_some());
    assert!(second_msg.is_some());
    Ok(())
}

#[tokio::test]
async fn webhook_partition_chat_only_shares_session_across_users() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook =
        build_telegram_webhook_app_with_partition(TelegramWebhookPartitionBuildRequest {
            bot_token: "fake-token".to_string(),
            allowed_users: vec!["*".to_string()],
            allowed_groups: vec![],
            admin_users: vec!["*".to_string()],
            webhook_path: "/telegram/webhook".to_string(),
            secret_token: None,
            dedup_config: WebhookDedupConfig {
                backend: WebhookDedupBackend::Memory,
                ttl_secs: 600,
            },
            session_partition: TelegramSessionPartition::ChatOnly,
            tx,
        })?;

    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(40_001, 81, -200_123, 888, None),
            None,
        )
        .await?,
        StatusCode::OK
    );
    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(40_002, 82, -200_123, 999, None),
            None,
        )
        .await?,
        StatusCode::OK
    );

    let first = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(first) = first else {
        return Err(anyhow!("first webhook message"));
    };
    let second = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(second) = second else {
        return Err(anyhow!("second webhook message"));
    };

    assert_eq!(first.session_key, "-200123");
    assert_eq!(second.session_key, "-200123");
    Ok(())
}

#[tokio::test]
async fn webhook_partition_chat_only_isolates_same_user_across_chats() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook =
        build_telegram_webhook_app_with_partition(TelegramWebhookPartitionBuildRequest {
            bot_token: "fake-token".to_string(),
            allowed_users: vec!["*".to_string()],
            allowed_groups: vec![],
            admin_users: vec!["*".to_string()],
            webhook_path: "/telegram/webhook".to_string(),
            secret_token: None,
            dedup_config: WebhookDedupConfig {
                backend: WebhookDedupBackend::Memory,
                ttl_secs: 600,
            },
            session_partition: TelegramSessionPartition::ChatOnly,
            tx,
        })?;

    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(40_101, 83, -200_123, 888, None),
            None,
        )
        .await?,
        StatusCode::OK
    );
    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(40_102, 84, -200_124, 888, None),
            None,
        )
        .await?,
        StatusCode::OK
    );

    let first = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(first) = first else {
        return Err(anyhow!("first webhook message"));
    };
    let second = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(second) = second else {
        return Err(anyhow!("second webhook message"));
    };

    assert_eq!(first.session_key, "-200123");
    assert_eq!(second.session_key, "-200124");
    assert_ne!(first.session_key, second.session_key);
    Ok(())
}

#[tokio::test]
async fn webhook_partition_chat_user_isolates_users() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook =
        build_telegram_webhook_app_with_partition(TelegramWebhookPartitionBuildRequest {
            bot_token: "fake-token".to_string(),
            allowed_users: vec!["*".to_string()],
            allowed_groups: vec![],
            admin_users: vec!["*".to_string()],
            webhook_path: "/telegram/webhook".to_string(),
            secret_token: None,
            dedup_config: WebhookDedupConfig {
                backend: WebhookDedupBackend::Memory,
                ttl_secs: 600,
            },
            session_partition: TelegramSessionPartition::ChatUser,
            tx,
        })?;

    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(41_001, 91, -200_123, 888, None),
            None,
        )
        .await?,
        StatusCode::OK
    );
    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(41_002, 92, -200_123, 999, None),
            None,
        )
        .await?,
        StatusCode::OK
    );

    let first = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(first) = first else {
        return Err(anyhow!("first webhook message"));
    };
    let second = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(second) = second else {
        return Err(anyhow!("second webhook message"));
    };

    assert_ne!(first.session_key, second.session_key);
    assert!(first.session_key.starts_with("-200123:"));
    assert!(second.session_key.starts_with("-200123:"));
    Ok(())
}

#[tokio::test]
async fn webhook_partition_chat_thread_user_isolates_topics() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(8);
    let webhook =
        build_telegram_webhook_app_with_partition(TelegramWebhookPartitionBuildRequest {
            bot_token: "fake-token".to_string(),
            allowed_users: vec!["*".to_string()],
            allowed_groups: vec![],
            admin_users: vec!["*".to_string()],
            webhook_path: "/telegram/webhook".to_string(),
            secret_token: None,
            dedup_config: WebhookDedupConfig {
                backend: WebhookDedupBackend::Memory,
                ttl_secs: 600,
            },
            session_partition: TelegramSessionPartition::ChatThreadUser,
            tx,
        })?;

    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(42_001, 101, -200_123, 888, Some(11)),
            None,
        )
        .await?,
        StatusCode::OK
    );
    assert_eq!(
        post_update(
            webhook.app.clone(),
            &webhook.path,
            sample_update_with_identity(42_002, 102, -200_123, 888, Some(22)),
            None,
        )
        .await?,
        StatusCode::OK
    );

    let first = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(first) = first else {
        return Err(anyhow!("first webhook message"));
    };
    let second = tokio::time::timeout(Duration::from_millis(250), rx.recv()).await?;
    let Some(second) = second else {
        return Err(anyhow!("second webhook message"));
    };

    assert_ne!(first.session_key, second.session_key);
    assert_eq!(first.recipient, "-200123:11");
    assert_eq!(second.recipient, "-200123:22");
    Ok(())
}
