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
