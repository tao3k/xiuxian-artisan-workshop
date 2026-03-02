//! MCP pool core helper tests.

use xiuxian_llm::mcp::{McpPoolConnectConfig, connect_pool_clients_with_retry};

#[tokio::test]
async fn connect_pool_clients_with_retry_rejects_zero_pool_size() {
    let cfg = McpPoolConnectConfig {
        pool_size: 0,
        ..McpPoolConnectConfig::default()
    };

    let result = connect_pool_clients_with_retry("http://127.0.0.1:65535/mcp", cfg).await;

    match result {
        Ok(_) => panic!("pool size 0 should be rejected before connect"),
        Err(error) => assert!(
            error
                .to_string()
                .contains("pool_size must be greater than 0"),
            "unexpected error: {error}"
        ),
    }
}
