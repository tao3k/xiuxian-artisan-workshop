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

use axum::{Json, extract::Multipart, extract::State, http::StatusCode};

use super::{MockTelegramUploadState, UploadCall};

pub(super) async fn handle_upload_media_group(
    State(state): State<MockTelegramUploadState>,
    mut multipart: Multipart,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut field_names = Vec::new();
    let mut text_fields = serde_json::Map::new();
    let mut media_json = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let Some(name) = field.name().map(str::to_string) else {
            continue;
        };

        if matches!(name.as_str(), "chat_id" | "message_thread_id" | "media")
            && let Ok(value) = field.text().await
        {
            if name == "media" {
                media_json = serde_json::from_str::<serde_json::Value>(&value).ok();
            } else {
                text_fields.insert(name.clone(), serde_json::json!(value));
            }
        }

        field_names.push(name);
    }

    let parse_mode = media_json
        .as_ref()
        .and_then(serde_json::Value::as_array)
        .and_then(|media| media.first())
        .and_then(|first| first.get("parse_mode"))
        .and_then(serde_json::Value::as_str);
    if parse_mode == Some("MarkdownV2") {
        let mut first_markdown_error = state.first_markdown_error.lock().await;
        if let Some(description) = first_markdown_error.take() {
            state.calls.lock().await.push(UploadCall {
                method: "sendMediaGroup".to_string(),
                field_names: field_names.clone(),
                text_fields: text_fields.clone(),
                media_json: media_json.clone(),
            });
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "ok": false,
                    "error_code": 400,
                    "description": description
                })),
            );
        }
    }

    state.calls.lock().await.push(UploadCall {
        method: "sendMediaGroup".to_string(),
        field_names: field_names.clone(),
        text_fields: text_fields.clone(),
        media_json: media_json.clone(),
    });
    *state.field_names.lock().await = field_names;
    *state.text_fields.lock().await = text_fields;
    *state.media_json.lock().await = media_json;

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}
