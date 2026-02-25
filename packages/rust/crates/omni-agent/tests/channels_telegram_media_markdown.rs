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

mod telegram_media_support;

use anyhow::Result;
use omni_agent::{Channel, TelegramChannel};

use telegram_media_support::{
    spawn_mock_telegram_media_api, spawn_mock_telegram_media_api_with_markdown_error,
};

#[tokio::test]
async fn telegram_media_group_caption_markdown_preserves_fenced_code_language_identifier()
-> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_media_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );
    let plain_caption = "```Rust\nlet value = a_b * 2;\n```";

    channel
        .send(
            &format!(
                "{plain_caption} [IMAGE:https://example.com/a.png][DOCUMENT:https://example.com/b.pdf]"
            ),
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "sendMediaGroup");

    let first_media = calls[0]
        .payload
        .get("media")
        .and_then(serde_json::Value::as_array)
        .and_then(|media| media.first())
        .cloned()
        .unwrap_or_default();

    assert_eq!(
        first_media
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("MarkdownV2")
    );
    assert_eq!(
        first_media
            .get("caption")
            .and_then(serde_json::Value::as_str),
        Some("```rust\nlet value = a_b * 2;\n```")
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_media_group_caption_markdown_fallback_keeps_original_fenced_code_text()
-> Result<()> {
    let Some((api_base, state, handle)) =
        spawn_mock_telegram_media_api_with_markdown_error(Some("can't parse entities")).await?
    else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );
    let plain_caption = "```Rust\nlet value = a_b * 2;\n```";

    channel
        .send(
            &format!(
                "{plain_caption} [IMAGE:https://example.com/a.png][DOCUMENT:https://example.com/b.pdf]"
            ),
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].method, "sendMediaGroup");
    assert_eq!(calls[1].method, "sendMediaGroup");

    let first_media = calls[0]
        .payload
        .get("media")
        .and_then(serde_json::Value::as_array)
        .and_then(|media| media.first())
        .cloned()
        .unwrap_or_default();
    let second_media = calls[1]
        .payload
        .get("media")
        .and_then(serde_json::Value::as_array)
        .and_then(|media| media.first())
        .cloned()
        .unwrap_or_default();

    assert_eq!(
        first_media
            .get("caption")
            .and_then(serde_json::Value::as_str),
        Some("```rust\nlet value = a_b * 2;\n```")
    );
    assert!(
        second_media.get("parse_mode").is_none(),
        "plain fallback media-group payload should not include parse_mode"
    );
    assert_eq!(
        second_media
            .get("caption")
            .and_then(serde_json::Value::as_str),
        Some(plain_caption)
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_media_group_caption_markdown_drops_unsupported_fenced_code_language_identifier()
-> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_media_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );
    let plain_caption = "```foo/bar\nlet value = 1;\n```";

    channel
        .send(
            &format!(
                "{plain_caption} [IMAGE:https://example.com/a.png][DOCUMENT:https://example.com/b.pdf]"
            ),
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "sendMediaGroup");

    let first_media = calls[0]
        .payload
        .get("media")
        .and_then(serde_json::Value::as_array)
        .and_then(|media| media.first())
        .cloned()
        .unwrap_or_default();
    assert_eq!(
        first_media
            .get("caption")
            .and_then(serde_json::Value::as_str),
        Some("```\nlet value = 1;\n```")
    );

    handle.abort();
    Ok(())
}
