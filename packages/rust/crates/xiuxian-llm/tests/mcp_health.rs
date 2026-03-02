//! MCP health endpoint derivation tests.

use xiuxian_llm::mcp::derive_health_url;

#[test]
fn derive_health_url_handles_streamable_suffixes() {
    assert_eq!(
        derive_health_url("http://127.0.0.1:3002/sse"),
        Some("http://127.0.0.1:3002/health".to_string())
    );
    assert_eq!(
        derive_health_url("http://127.0.0.1:3002/messages"),
        Some("http://127.0.0.1:3002/health".to_string())
    );
    assert_eq!(
        derive_health_url("http://127.0.0.1:3002/mcp"),
        Some("http://127.0.0.1:3002/health".to_string())
    );
}

#[test]
fn derive_health_url_handles_generic_base_and_empty_input() {
    assert_eq!(
        derive_health_url("http://127.0.0.1:3002"),
        Some("http://127.0.0.1:3002/health".to_string())
    );
    assert_eq!(derive_health_url("  "), None);
}
