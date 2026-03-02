use anyhow::{Context, Result};

use crate::config::AgentConfig;
use crate::mcp::{McpClientPool, McpPoolConnectConfig, connect_pool};

const NON_STRICT_STARTUP_HANDSHAKE_TIMEOUT_SECS: u64 = 5;
const NON_STRICT_STARTUP_CONNECT_RETRIES: u32 = 1;

pub(super) async fn connect_mcp_pool_if_configured(
    config: &AgentConfig,
) -> Result<Option<McpClientPool>> {
    let Some(url) = config
        .mcp_servers
        .iter()
        .find(|server| server.url.is_some())
        .and_then(|server| server.url.as_deref())
    else {
        return Ok(None);
    };

    let strict_startup = config.mcp_strict_startup;
    let connect_config = startup_connect_config(config, strict_startup);
    match connect_pool(url, connect_config).await {
        Ok(pool) => Ok(Some(pool)),
        Err(error) if strict_startup => Err(error).with_context(|| {
            format!(
                "strict MCP startup connect failed (url={url}, retries={}, handshake_timeout_secs={})",
                connect_config.connect_retries, connect_config.handshake_timeout_secs
            )
        }),
        Err(error) => {
            tracing::warn!(
                event = "agent.mcp.startup.connect_failed",
                strict_startup = false,
                continue_startup = true,
                url,
                pool_size = connect_config.pool_size,
                retries = connect_config.connect_retries,
                handshake_timeout_secs = connect_config.handshake_timeout_secs,
                connect_retry_backoff_ms = connect_config.connect_retry_backoff_ms,
                error = %error,
                "mcp startup connect failed in non-strict mode; continuing without MCP tools"
            );
            Ok(None)
        }
    }
}

fn startup_connect_config(config: &AgentConfig, strict_startup: bool) -> McpPoolConnectConfig {
    if strict_startup {
        return McpPoolConnectConfig {
            pool_size: config.mcp_pool_size,
            handshake_timeout_secs: config.mcp_handshake_timeout_secs,
            connect_retries: config.mcp_connect_retries,
            connect_retry_backoff_ms: config.mcp_connect_retry_backoff_ms,
            tool_timeout_secs: config.mcp_tool_timeout_secs,
            list_tools_cache_ttl_ms: config.mcp_list_tools_cache_ttl_ms,
        };
    }

    McpPoolConnectConfig {
        pool_size: config.mcp_pool_size,
        handshake_timeout_secs: config
            .mcp_handshake_timeout_secs
            .clamp(1, NON_STRICT_STARTUP_HANDSHAKE_TIMEOUT_SECS),
        connect_retries: config
            .mcp_connect_retries
            .clamp(1, NON_STRICT_STARTUP_CONNECT_RETRIES),
        connect_retry_backoff_ms: config.mcp_connect_retry_backoff_ms.max(1),
        tool_timeout_secs: config.mcp_tool_timeout_secs,
        list_tools_cache_ttl_ms: config.mcp_list_tools_cache_ttl_ms,
    }
}
