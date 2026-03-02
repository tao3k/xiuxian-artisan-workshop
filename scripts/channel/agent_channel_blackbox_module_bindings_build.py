#!/usr/bin/env python3
"""Binding-object construction for blackbox module loader."""

from __future__ import annotations

from typing import Any


def build_bindings(modules: dict[str, Any], *, bindings_cls: type[Any]) -> Any:
    """Build a typed bindings object from resolved module map."""
    resolver_module = modules["resolver_module"]
    log_io_module = modules["log_io_module"]
    config_module = modules["config_module"]
    parsing_module = modules["parsing_module"]
    runtime_module = modules["runtime_module"]
    models_module = modules["models_module"]
    constants_module = modules["constants_module"]
    entry_flow_module = modules["entry_flow_module"]
    log_bindings_module = modules["log_bindings_module"]
    session_exports_module = modules["session_exports_module"]
    session_keys_module = modules["session_keys_module"]
    session_bindings_module = modules["session_bindings_module"]
    http_module = modules["http_module"]
    entry_bindings_module = modules["entry_bindings_module"]

    return bindings_cls(
        default_telegram_webhook_url=resolver_module.default_telegram_webhook_url,
        normalize_telegram_session_partition_mode=(
            resolver_module.normalize_telegram_session_partition_mode
        ),
        session_ids_from_runtime_log=resolver_module.session_ids_from_runtime_log,
        telegram_webhook_secret_token=resolver_module.telegram_webhook_secret_token,
        username_from_runtime_log=resolver_module.username_from_runtime_log,
        username_from_settings=resolver_module.username_from_settings,
        shared_log_cursor_cls=log_io_module.LogCursor,
        shared_init_log_cursor=log_io_module.init_log_cursor,
        shared_read_new_log_lines_with_cursor=log_io_module.read_new_log_lines_with_cursor,
        shared_tail_log_lines=log_io_module.tail_log_lines,
        config_module=config_module,
        parsing_module=parsing_module,
        runtime_module=runtime_module,
        probe_config_cls=models_module.ProbeConfig,
        constants_module=constants_module,
        entry_flow_module=entry_flow_module,
        log_bindings_module=log_bindings_module,
        session_exports_module=session_exports_module,
        session_keys_module=session_keys_module,
        session_bindings_module=session_bindings_module,
        http_module=http_module,
        entry_bindings_module=entry_bindings_module,
        error_patterns=tuple(constants_module.ERROR_PATTERNS),
        mcp_observability_events=tuple(constants_module.MCP_OBSERVABILITY_EVENTS),
        mcp_waiting_events=frozenset(constants_module.MCP_WAITING_EVENTS),
        target_session_scope_placeholder=constants_module.TARGET_SESSION_SCOPE_PLACEHOLDER,
        telegram_session_scope_prefix=constants_module.TELEGRAM_SESSION_SCOPE_PREFIX,
        discord_session_scope_prefix=constants_module.DISCORD_SESSION_SCOPE_PREFIX,
    )
