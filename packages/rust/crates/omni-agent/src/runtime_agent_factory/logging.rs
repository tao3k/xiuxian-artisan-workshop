use super::types::{McpRuntimeOptions, MemoryRuntimeOptions, SessionRuntimeOptions};

pub(super) fn log_runtime_agent_options(
    mcp: &McpRuntimeOptions,
    session: &SessionRuntimeOptions,
    memory: &MemoryRuntimeOptions,
) {
    let memory_config = &memory.config;
    tracing::info!(
        mcp_pool_size = mcp.pool_size,
        mcp_handshake_timeout_secs = mcp.handshake_timeout_secs,
        mcp_connect_retries = mcp.connect_retries,
        mcp_strict_startup = mcp.strict_startup,
        mcp_connect_retry_backoff_ms = mcp.connect_retry_backoff_ms,
        mcp_tool_timeout_secs = mcp.tool_timeout_secs,
        mcp_list_tools_cache_ttl_ms = mcp.list_tools_cache_ttl_ms,
        window_max_turns = ?session.window_max_turns,
        consolidation_threshold_turns = ?session.consolidation_threshold_turns,
        consolidation_take_turns = session.consolidation_take_turns,
        consolidation_async = session.consolidation_async,
        context_budget_tokens = ?session.context_budget_tokens,
        context_budget_reserve_tokens = session.context_budget_reserve_tokens,
        context_budget_strategy = session.context_budget_strategy.as_str(),
        summary_max_segments = session.summary_max_segments,
        summary_max_chars = session.summary_max_chars,
        memory_embedding_backend = memory_config
            .embedding_backend
            .as_deref()
            .unwrap_or(memory.embedding_backend_mode.as_str()),
        memory_embedding_model = memory_config.embedding_model.as_deref().unwrap_or(""),
        memory_embedding_dim = memory_config.embedding_dim,
        memory_embedding_timeout_ms = ?memory_config.embedding_timeout_ms,
        memory_embedding_timeout_cooldown_ms = ?memory_config.embedding_timeout_cooldown_ms,
        memory_embedding_base_url = memory_config.embedding_base_url.as_deref().unwrap_or(""),
        memory_persistence_backend = %memory_config.persistence_backend,
        memory_persistence_strict_startup = ?memory_config.persistence_strict_startup,
        memory_recall_credit_enabled = memory_config.recall_credit_enabled,
        memory_recall_credit_max_candidates = memory_config.recall_credit_max_candidates,
        memory_decay_enabled = memory_config.decay_enabled,
        memory_decay_every_turns = memory_config.decay_every_turns,
        memory_decay_factor = memory_config.decay_factor,
        memory_gate_promote_threshold = memory_config.gate_promote_threshold,
        memory_gate_obsolete_threshold = memory_config.gate_obsolete_threshold,
        memory_gate_promote_min_usage = memory_config.gate_promote_min_usage,
        memory_gate_obsolete_min_usage = memory_config.gate_obsolete_min_usage,
        memory_gate_promote_failure_rate_ceiling = memory_config.gate_promote_failure_rate_ceiling,
        memory_gate_obsolete_failure_rate_floor = memory_config.gate_obsolete_failure_rate_floor,
        memory_gate_promote_min_ttl_score = memory_config.gate_promote_min_ttl_score,
        memory_gate_obsolete_max_ttl_score = memory_config.gate_obsolete_max_ttl_score,
        memory_stream_consumer_enabled = memory_config.stream_consumer_enabled,
        memory_stream_name = %memory_config.stream_name,
        memory_stream_consumer_group = %memory_config.stream_consumer_group,
        memory_stream_consumer_name_prefix = %memory_config.stream_consumer_name_prefix,
        memory_stream_consumer_batch_size = memory_config.stream_consumer_batch_size,
        memory_stream_consumer_block_ms = memory_config.stream_consumer_block_ms,
        memory_path = %memory_config.path,
        "telegram runtime session window settings"
    );
}
