#!/usr/bin/env python3
"""Dependency loader for dedup probe entry script."""

from __future__ import annotations

import importlib
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class DedupProbeModuleBindings:
    """Resolved sibling modules and resolver callables for dedup probe."""

    default_telegram_webhook_url: Any
    session_ids_from_runtime_log: Any
    telegram_webhook_secret_token: Any
    username_from_runtime_log: Any
    username_from_settings: Any
    shared_log_cursor_cls: Any
    shared_init_log_cursor: Any
    shared_read_new_log_lines_with_cursor: Any
    flow_module: Any


def load_module_bindings(caller_file: str) -> DedupProbeModuleBindings:
    """Load all sibling modules required by `test_omni_agent_dedup_events.py`."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    resolver_module = load_sibling_module(
        module_name="config_resolver",
        file_name="config_resolver.py",
        caller_file=caller_file,
        error_context="resolver module",
    )
    log_io_module = load_sibling_module(
        module_name="log_io",
        file_name="log_io.py",
        caller_file=caller_file,
        error_context="shared log I/O helpers",
    )
    flow_module = load_sibling_module(
        module_name="dedup_probe_flow",
        file_name="dedup_probe_flow.py",
        caller_file=caller_file,
        error_context="dedup probe flow helpers",
    )

    return DedupProbeModuleBindings(
        default_telegram_webhook_url=resolver_module.default_telegram_webhook_url,
        session_ids_from_runtime_log=resolver_module.session_ids_from_runtime_log,
        telegram_webhook_secret_token=resolver_module.telegram_webhook_secret_token,
        username_from_runtime_log=resolver_module.username_from_runtime_log,
        username_from_settings=resolver_module.username_from_settings,
        shared_log_cursor_cls=log_io_module.LogCursor,
        shared_init_log_cursor=log_io_module.init_log_cursor,
        shared_read_new_log_lines_with_cursor=log_io_module.read_new_log_lines_with_cursor,
        flow_module=flow_module,
    )
