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

use crate::channels::telegram::{
    TelegramControlCommandPolicy, WebhookDedupBackend, WebhookDedupConfig,
};

use super::super::run_telegram_webhook_with_control_command_policy;
use super::build_agent;

#[tokio::test]
async fn runtime_webhook_requires_non_empty_secret_token() -> Result<()> {
    let agent = build_agent().await?;
    let error = run_telegram_webhook_with_control_command_policy(
        agent,
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::default(),
        "127.0.0.1:0",
        "/telegram/webhook",
        None,
        WebhookDedupConfig {
            backend: WebhookDedupBackend::Memory,
            ttl_secs: 600,
        },
    )
    .await
    .expect_err("missing webhook secret should fail before starting runtime");

    assert!(
        error
            .to_string()
            .contains("requires a non-empty secret token"),
        "unexpected error: {error}"
    );
    Ok(())
}
