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

use crate::session::ChatMessage;

use super::types::ChatCompletionRequest;

fn sample_user_message() -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content: Some("hello".to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    }
}

#[test]
fn http_request_omits_max_tokens_when_not_set() {
    let request = ChatCompletionRequest {
        model: "MiniMax-M2.5".to_string(),
        messages: vec![sample_user_message()],
        max_tokens: None,
        tools: None,
        tool_choice: None,
    };
    let value = serde_json::to_value(&request).expect("serialize chat request");
    assert!(value.get("max_tokens").is_none());
}

#[test]
fn http_request_includes_max_tokens_when_set() {
    let request = ChatCompletionRequest {
        model: "MiniMax-M2.5".to_string(),
        messages: vec![sample_user_message()],
        max_tokens: Some(1024),
        tools: None,
        tool_choice: None,
    };
    let value = serde_json::to_value(&request).expect("serialize chat request");
    assert_eq!(value.get("max_tokens"), Some(&serde_json::json!(1024)));
}
