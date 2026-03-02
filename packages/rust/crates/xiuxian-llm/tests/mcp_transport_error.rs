//! MCP transport error classification tests.

use anyhow::anyhow;
use xiuxian_llm::mcp::{classify_transport_error, should_retry_transport_error};

#[test]
fn classify_transport_error_marks_embedding_timeout_non_retryable() {
    let error = anyhow!("Embedding timed out after 5s");
    let class = classify_transport_error(&error);
    assert_eq!(class.kind, "tool_embedding_timeout");
    assert!(!class.retryable);
    assert!(!should_retry_transport_error(&error));
}

#[test]
fn classify_transport_error_marks_connection_refused_retryable() {
    let error = anyhow!("transport send error: connection refused");
    let class = classify_transport_error(&error);
    assert_eq!(class.kind, "transport_send");
    assert!(class.retryable);
    assert!(should_retry_transport_error(&error));
}

#[test]
fn classify_transport_error_marks_unknown_non_retryable() {
    let error = anyhow!("unexpected parse error");
    let class = classify_transport_error(&error);
    assert_eq!(class.kind, "non_transport");
    assert!(!class.retryable);
    assert!(!should_retry_transport_error(&error));
}
