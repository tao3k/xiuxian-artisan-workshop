#!/usr/bin/env python3
"""Module-resolution helpers for blackbox bindings loader."""

from __future__ import annotations

import importlib
from typing import Any


def resolve_modules(caller_file: str) -> dict[str, Any]:
    """Resolve all sibling modules used by blackbox entry script."""
    load_sibling_module = importlib.import_module("module_loader").load_sibling_module

    def _load_sibling(module_name: str, file_name: str, error_context: str) -> Any:
        return load_sibling_module(
            module_name=module_name,
            file_name=file_name,
            caller_file=caller_file,
            error_context=error_context,
        )

    return {
        "resolver_module": _load_sibling(
            "config_resolver", "config_resolver.py", "resolver module"
        ),
        "log_io_module": _load_sibling("log_io", "log_io.py", "shared log I/O helpers"),
        "config_module": _load_sibling(
            "agent_channel_blackbox_config",
            "agent_channel_blackbox_config.py",
            "blackbox config helpers",
        ),
        "parsing_module": _load_sibling(
            "agent_channel_blackbox_parsing",
            "agent_channel_blackbox_parsing.py",
            "blackbox parsing helpers",
        ),
        "runtime_module": _load_sibling(
            "agent_channel_blackbox_runtime",
            "agent_channel_blackbox_runtime.py",
            "blackbox runtime helpers",
        ),
        "models_module": _load_sibling(
            "agent_channel_blackbox_models",
            "agent_channel_blackbox_models.py",
            "blackbox models",
        ),
        "constants_module": _load_sibling(
            "agent_channel_blackbox_constants",
            "agent_channel_blackbox_constants.py",
            "blackbox constants",
        ),
        "entry_flow_module": _load_sibling(
            "agent_channel_blackbox_entry_flow",
            "agent_channel_blackbox_entry_flow.py",
            "blackbox entry flow bindings",
        ),
        "log_bindings_module": _load_sibling(
            "agent_channel_blackbox_log_bindings",
            "agent_channel_blackbox_log_bindings.py",
            "blackbox log bindings",
        ),
        "session_exports_module": _load_sibling(
            "agent_channel_blackbox_session_exports",
            "agent_channel_blackbox_session_exports.py",
            "blackbox session export bindings",
        ),
        "session_keys_module": _load_sibling(
            "telegram_session_keys",
            "telegram_session_keys.py",
            "telegram session key helpers",
        ),
        "session_bindings_module": _load_sibling(
            "agent_channel_blackbox_session_bindings",
            "agent_channel_blackbox_session_bindings.py",
            "blackbox session bindings",
        ),
        "http_module": _load_sibling(
            "agent_channel_blackbox_http",
            "agent_channel_blackbox_http.py",
            "blackbox http helpers",
        ),
        "entry_bindings_module": _load_sibling(
            "agent_channel_blackbox_entry_bindings",
            "agent_channel_blackbox_entry_bindings.py",
            "blackbox entry bindings",
        ),
    }
