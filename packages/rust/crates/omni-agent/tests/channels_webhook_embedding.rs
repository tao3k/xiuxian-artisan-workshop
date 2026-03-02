//! Test coverage for omni-agent behavior.

use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use omni_agent::{WebhookDedupBackend, WebhookDedupConfig, build_telegram_webhook_app};
use tokio::sync::mpsc;
use tower::util::ServiceExt;

#[tokio::test]
async fn webhook_router_exposes_embedding_endpoints() -> Result<()> {
    let (tx, _rx) = mpsc::channel(8);
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

    let embed = webhook
        .app
        .clone()
        .oneshot(
            Request::post("/embed")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"text":"   "}"#))?,
        )
        .await?;
    assert_eq!(embed.status(), StatusCode::BAD_REQUEST);

    let embed_single = webhook
        .app
        .clone()
        .oneshot(
            Request::post("/embed/single")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"text":"   "}"#))?,
        )
        .await?;
    assert_eq!(embed_single.status(), StatusCode::BAD_REQUEST);

    let embed_batch = webhook
        .app
        .clone()
        .oneshot(
            Request::post("/embed/batch")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"texts":[]}"#))?,
        )
        .await?;
    assert_eq!(embed_batch.status(), StatusCode::BAD_REQUEST);

    let openai = webhook
        .app
        .oneshot(
            Request::post("/v1/embeddings")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"input":123}"#))?,
        )
        .await?;
    assert_eq!(openai.status(), StatusCode::BAD_REQUEST);

    Ok(())
}
