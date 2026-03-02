#!/usr/bin/env python3
"""Entry-flow bindings for channel blackbox probe."""

from __future__ import annotations

import importlib
from typing import Any

_RUNTIME = importlib.import_module("agent_channel_blackbox_entry_flow_runtime")
run_probe = _RUNTIME.run_probe
run_main = _RUNTIME.run_main


def parse_args(
    *,
    entry_bindings_module: Any,
    config_module: Any,
    default_telegram_webhook_url_fn: Any,
    target_session_scope_placeholder: str,
) -> Any:
    """Parse CLI args for the blackbox probe."""
    return entry_bindings_module.parse_args(
        config_module=config_module,
        default_telegram_webhook_url_fn=default_telegram_webhook_url_fn,
        target_session_scope_placeholder=target_session_scope_placeholder,
    )


def build_config(
    args: Any,
    *,
    entry_bindings_module: Any,
    config_module: Any,
    probe_config_cls: Any,
    session_ids_from_runtime_log_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    parse_expected_field_fn: Any,
    parse_allow_chat_ids_fn: Any,
    normalize_session_partition_fn: Any,
    telegram_webhook_secret_token_fn: Any,
) -> Any:
    """Build normalized probe configuration from parsed args."""
    return entry_bindings_module.build_config(
        args,
        config_module=config_module,
        probe_config_cls=probe_config_cls,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
        parse_expected_field_fn=parse_expected_field_fn,
        parse_allow_chat_ids_fn=parse_allow_chat_ids_fn,
        normalize_session_partition_fn=normalize_session_partition_fn,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token_fn,
    )
