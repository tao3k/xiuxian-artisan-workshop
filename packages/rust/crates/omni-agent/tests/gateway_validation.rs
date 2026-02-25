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

use axum::http::StatusCode;
use omni_agent::{MessageRequest, validate_message_request};

#[test]
fn validate_rejects_empty_session_id() {
    let body = MessageRequest {
        session_id: String::new(),
        message: "hi".to_string(),
    };
    let result = validate_message_request(&body);
    assert!(result.is_err());
    assert_eq!(result.expect_err("err").0, StatusCode::BAD_REQUEST);
}

#[test]
fn validate_rejects_empty_message() {
    let body = MessageRequest {
        session_id: "s1".to_string(),
        message: "  ".to_string(),
    };
    let result = validate_message_request(&body);
    assert!(result.is_err());
    assert_eq!(result.expect_err("err").0, StatusCode::BAD_REQUEST);
}

#[test]
fn validate_accepts_trimmed_values() {
    let body = MessageRequest {
        session_id: "  s1  ".to_string(),
        message: " hello ".to_string(),
    };
    let (session_id, message) = validate_message_request(&body).expect("ok");
    assert_eq!(session_id, "s1");
    assert_eq!(message, "hello");
}
