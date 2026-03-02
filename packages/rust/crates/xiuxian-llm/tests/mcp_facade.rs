//! MCP facade re-export and default-contract tests.

use xiuxian_llm::mcp::{
    McpClientPool, McpDiscoverCacheStatsSnapshot, McpPoolConnectConfig, McpServerTransportConfig,
    McpToolsListCacheStatsSnapshot, OmniMcpClient, call_tool_once, connect_pool,
    connect_pool_clients_with_retry, init_params_omni_server, list_tools_once,
    reconnect_pool_client_with_retry,
};

#[test]
fn mcp_facade_reexports_client_surface() {
    let _init = init_params_omni_server();
    let cfg = McpServerTransportConfig::StreamableHttp {
        url: "http://127.0.0.1:3002/mcp".to_string(),
        bearer_token_env_var: None,
    };
    let _client = OmniMcpClient::from_config(&cfg);
}

#[test]
fn mcp_pool_connect_config_default_is_stable() {
    let cfg = McpPoolConnectConfig::default();
    assert_eq!(cfg.pool_size, 4);
    assert_eq!(cfg.handshake_timeout_secs, 30);
    assert_eq!(cfg.connect_retries, 3);
    assert_eq!(cfg.connect_retry_backoff_ms, 1_000);
    assert_eq!(cfg.tool_timeout_secs, 180);
    assert_eq!(cfg.list_tools_cache_ttl_ms, 1_000);
}

#[test]
fn mcp_facade_reexports_pool_core_helpers() {
    fn touch<T>(_value: T) {}

    touch(connect_pool_clients_with_retry);
    touch(reconnect_pool_client_with_retry);
    touch(connect_pool);
    touch(list_tools_once);
    touch(call_tool_once);

    assert!(!std::any::type_name::<McpClientPool>().is_empty());
    assert!(!std::any::type_name::<McpToolsListCacheStatsSnapshot>().is_empty());
    assert!(!std::any::type_name::<McpDiscoverCacheStatsSnapshot>().is_empty());
}
