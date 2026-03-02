//! Test coverage for omni-agent behavior.

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
    let value = match serde_json::to_value(&request) {
        Ok(value) => value,
        Err(error) => panic!("serialize chat request: {error}"),
    };
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
    let value = match serde_json::to_value(&request) {
        Ok(value) => value,
        Err(error) => panic!("serialize chat request: {error}"),
    };
    assert_eq!(value.get("max_tokens"), Some(&serde_json::json!(1024)));
}
