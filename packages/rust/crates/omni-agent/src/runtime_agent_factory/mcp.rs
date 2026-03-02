use std::path::Path;

use anyhow::Result;
use omni_agent::{McpServerEntry, RuntimeSettings, load_mcp_config};

use crate::resolve::{
    parse_bool_from_env, parse_positive_u32_from_env, parse_positive_u64_from_env,
    parse_positive_usize_from_env,
};

use super::types::McpRuntimeOptions;

pub(super) fn resolve_runtime_mcp_servers(mcp_config_path: &Path) -> Result<Vec<McpServerEntry>> {
    Ok(load_mcp_config(mcp_config_path)?
        .into_iter()
        .filter(|entry| entry.url.is_some())
        .collect::<Vec<_>>())
}

pub(super) fn resolve_runtime_mcp_options(runtime_settings: &RuntimeSettings) -> McpRuntimeOptions {
    McpRuntimeOptions {
        pool_size: parse_positive_usize_from_env("OMNI_AGENT_MCP_POOL_SIZE")
            .or(runtime_settings.mcp.pool_size.filter(|value| *value > 0))
            .unwrap_or(4),
        handshake_timeout_secs: parse_positive_u64_from_env(
            "OMNI_AGENT_MCP_HANDSHAKE_TIMEOUT_SECS",
        )
        .or(runtime_settings
            .mcp
            .handshake_timeout_secs
            .filter(|value| *value > 0))
        .unwrap_or(30),
        connect_retries: parse_positive_u32_from_env("OMNI_AGENT_MCP_CONNECT_RETRIES")
            .or(runtime_settings
                .mcp
                .connect_retries
                .filter(|value| *value > 0))
            .unwrap_or(3),
        strict_startup: parse_bool_from_env("OMNI_AGENT_MCP_STRICT_STARTUP")
            .or(runtime_settings.mcp.strict_startup)
            .unwrap_or(true),
        connect_retry_backoff_ms: parse_positive_u64_from_env(
            "OMNI_AGENT_MCP_CONNECT_RETRY_BACKOFF_MS",
        )
        .or(runtime_settings
            .mcp
            .connect_retry_backoff_ms
            .filter(|value| *value > 0))
        .unwrap_or(1_000),
        tool_timeout_secs: parse_positive_u64_from_env("OMNI_AGENT_MCP_TOOL_TIMEOUT_SECS")
            .or(runtime_settings
                .mcp
                .tool_timeout_secs
                .filter(|value| *value > 0))
            .unwrap_or(180),
        list_tools_cache_ttl_ms: parse_positive_u64_from_env(
            "OMNI_AGENT_MCP_LIST_TOOLS_CACHE_TTL_MS",
        )
        .or(runtime_settings
            .mcp
            .list_tools_cache_ttl_ms
            .filter(|value| *value > 0))
        .unwrap_or(1_000),
    }
}
