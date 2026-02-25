use std::path::Path;

use anyhow::Result;
#[cfg(test)]
use omni_agent::LITELLM_DEFAULT_URL;
#[cfg(test)]
use omni_agent::McpServerEntry;
use omni_agent::{Agent, AgentConfig, RuntimeSettings};

mod inference;
mod logging;
mod mcp;
mod memory;
mod session;
mod shared;
mod types;

#[cfg(test)]
use inference::{
    parse_embedding_backend_mode, resolve_inference_url, resolve_runtime_embedding_backend_mode,
    resolve_runtime_embedding_base_url, validate_inference_url_origin,
};
use inference::{resolve_runtime_inference_url, resolve_runtime_model};
use logging::log_runtime_agent_options;
use mcp::{resolve_runtime_mcp_options, resolve_runtime_mcp_servers};
use memory::resolve_runtime_memory_options;
use session::resolve_runtime_session_options;
#[cfg(test)]
use types::RuntimeEmbeddingBackendMode;

pub(crate) async fn build_agent(
    mcp_config_path: &Path,
    runtime_settings: &RuntimeSettings,
) -> Result<Agent> {
    let mcp_servers = resolve_runtime_mcp_servers(mcp_config_path)?;
    let inference_url = resolve_runtime_inference_url(runtime_settings, &mcp_servers)?;
    let model = resolve_runtime_model(runtime_settings);
    let mcp = resolve_runtime_mcp_options(runtime_settings);
    let session = resolve_runtime_session_options(runtime_settings)?;
    let memory = resolve_runtime_memory_options(runtime_settings);

    log_runtime_agent_options(&mcp, &session, &memory);

    let config = AgentConfig {
        inference_url,
        model,
        api_key: None,
        mcp_servers,
        mcp_pool_size: mcp.pool_size,
        mcp_handshake_timeout_secs: mcp.handshake_timeout_secs,
        mcp_connect_retries: mcp.connect_retries,
        mcp_strict_startup: mcp.strict_startup,
        mcp_connect_retry_backoff_ms: mcp.connect_retry_backoff_ms,
        mcp_tool_timeout_secs: mcp.tool_timeout_secs,
        mcp_list_tools_cache_ttl_ms: mcp.list_tools_cache_ttl_ms,
        max_tool_rounds: session.max_tool_rounds,
        memory: Some(memory.config),
        window_max_turns: session.window_max_turns,
        consolidation_threshold_turns: session.consolidation_threshold_turns,
        consolidation_take_turns: session.consolidation_take_turns,
        consolidation_async: session.consolidation_async,
        context_budget_tokens: session.context_budget_tokens,
        context_budget_reserve_tokens: session.context_budget_reserve_tokens,
        context_budget_strategy: session.context_budget_strategy,
        summary_max_segments: session.summary_max_segments,
        summary_max_chars: session.summary_max_chars,
    };
    Agent::from_config(config).await
}

#[cfg(test)]
#[path = "../../tests/runtime_agent_factory/inference.rs"]
mod tests;
