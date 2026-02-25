#!/usr/bin/env python3
"""Dependency loader for agent channel blackbox entry script."""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class BlackboxModuleBindings:
    """Resolved sibling modules and helper callables for blackbox probes."""

    default_telegram_webhook_url: Any
    normalize_telegram_session_partition_mode: Any
    session_ids_from_runtime_log: Any
    telegram_webhook_secret_token: Any
    username_from_runtime_log: Any
    username_from_settings: Any
    shared_log_cursor_cls: Any
    shared_init_log_cursor: Any
    shared_read_new_log_lines_with_cursor: Any
    shared_tail_log_lines: Any
    config_module: Any
    parsing_module: Any
    runtime_module: Any
    probe_config_cls: Any
    constants_module: Any
    entry_flow_module: Any
    log_bindings_module: Any
    session_exports_module: Any
    session_keys_module: Any
    session_bindings_module: Any
    http_module: Any
    entry_bindings_module: Any
    error_patterns: tuple[str, ...]
    mcp_observability_events: tuple[str, ...]
    mcp_waiting_events: frozenset[str]
    target_session_scope_placeholder: str
    telegram_session_scope_prefix: str
    discord_session_scope_prefix: str


def load_module_bindings(caller_file: str) -> BlackboxModuleBindings:
    """Load all sibling modules required by `agent_channel_blackbox.py`."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    def _load_sibling(module_name: str, file_name: str, error_context: str) -> Any:
        return load_sibling_module(
            module_name=module_name,
            file_name=file_name,
            caller_file=caller_file,
            error_context=error_context,
        )

    resolver_module = _load_sibling("config_resolver", "config_resolver.py", "resolver module")
    log_io_module = _load_sibling("log_io", "log_io.py", "shared log I/O helpers")
    config_module = _load_sibling(
        "agent_channel_blackbox_config",
        "agent_channel_blackbox_config.py",
        "blackbox config helpers",
    )
    parsing_module = _load_sibling(
        "agent_channel_blackbox_parsing",
        "agent_channel_blackbox_parsing.py",
        "blackbox parsing helpers",
    )
    runtime_module = _load_sibling(
        "agent_channel_blackbox_runtime",
        "agent_channel_blackbox_runtime.py",
        "blackbox runtime helpers",
    )
    models_module = _load_sibling(
        "agent_channel_blackbox_models",
        "agent_channel_blackbox_models.py",
        "blackbox models",
    )
    constants_module = _load_sibling(
        "agent_channel_blackbox_constants",
        "agent_channel_blackbox_constants.py",
        "blackbox constants",
    )
    entry_flow_module = _load_sibling(
        "agent_channel_blackbox_entry_flow",
        "agent_channel_blackbox_entry_flow.py",
        "blackbox entry flow bindings",
    )
    log_bindings_module = _load_sibling(
        "agent_channel_blackbox_log_bindings",
        "agent_channel_blackbox_log_bindings.py",
        "blackbox log bindings",
    )
    session_exports_module = _load_sibling(
        "agent_channel_blackbox_session_exports",
        "agent_channel_blackbox_session_exports.py",
        "blackbox session export bindings",
    )
    session_keys_module = _load_sibling(
        "telegram_session_keys",
        "telegram_session_keys.py",
        "telegram session key helpers",
    )
    session_bindings_module = _load_sibling(
        "agent_channel_blackbox_session_bindings",
        "agent_channel_blackbox_session_bindings.py",
        "blackbox session bindings",
    )
    http_module = _load_sibling(
        "agent_channel_blackbox_http",
        "agent_channel_blackbox_http.py",
        "blackbox http helpers",
    )
    entry_bindings_module = _load_sibling(
        "agent_channel_blackbox_entry_bindings",
        "agent_channel_blackbox_entry_bindings.py",
        "blackbox entry bindings",
    )

    return BlackboxModuleBindings(
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
