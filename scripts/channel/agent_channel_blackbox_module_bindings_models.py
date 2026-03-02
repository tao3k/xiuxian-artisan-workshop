#!/usr/bin/env python3
"""Datamodels for agent channel blackbox module bindings."""

from __future__ import annotations

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
