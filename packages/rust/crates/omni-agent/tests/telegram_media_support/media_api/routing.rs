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

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use super::{
    MediaCall, MockTelegramMediaState, markdown_fallback::take_markdown_error_description,
};

pub(super) async fn handle_method(
    Path(method): Path<String>,
    State(state): State<MockTelegramMediaState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    if method == "sendMediaGroup" {
        let mut remaining_failures = state.fail_send_media_group_remaining.lock().await;
        if *remaining_failures > 0 {
            *remaining_failures -= 1;
            state.calls.lock().await.push(MediaCall { method, payload });
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "ok": false,
                    "error_code": 500,
                    "description": "internal error"
                })),
            );
        }
    }

    if let Some(description) = take_markdown_error_description(&payload, &state).await {
        state.calls.lock().await.push(MediaCall { method, payload });
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error_code": 400,
                "description": description
            })),
        );
    }

    state.calls.lock().await.push(MediaCall { method, payload });
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}
