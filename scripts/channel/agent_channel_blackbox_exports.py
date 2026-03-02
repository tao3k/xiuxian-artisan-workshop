#!/usr/bin/env python3
"""Compatibility export bindings for agent_channel_blackbox module."""

from __future__ import annotations

from typing import Any


def apply_compat_exports(
    namespace: dict[str, Any],
    *,
    modules: Any,
    parsing_module: Any,
    session_bindings_module: Any,
    session_exports_module: Any,
    telegram_prefix: str,
    discord_prefix: str,
) -> None:
    """Populate compatibility exports expected by blackbox tests/callers."""
    namespace["strip_ansi"] = parsing_module.strip_ansi
    namespace["extract_event_token"] = parsing_module.extract_event_token
    namespace["extract_session_key_token"] = parsing_module.extract_session_key_token
    namespace["parse_log_tokens"] = parsing_module.parse_log_tokens
    namespace["parse_expected_field"] = parsing_module.parse_expected_field
    namespace["parse_allow_chat_ids"] = parsing_module.parse_allow_chat_ids
    namespace["parse_command_reply_event_line"] = parsing_module.parse_command_reply_event_line
    namespace["parse_command_reply_json_summary_line"] = (
        parsing_module.parse_command_reply_json_summary_line
    )
    namespace["telegram_send_retry_grace_seconds"] = (
        parsing_module.telegram_send_retry_grace_seconds
    )

    # Backward-compatible aliases for existing test-kit imports.
    namespace["infer_ids_from_log"] = modules.session_ids_from_runtime_log
    namespace["infer_username_from_log"] = modules.username_from_runtime_log
    namespace["username_from_settings"] = modules.username_from_settings
    namespace["build_update_payload"] = modules.config_module.build_update_payload
    namespace["build_probe_message"] = modules.config_module.build_probe_message

    session_helpers = session_exports_module.build_session_helpers(
        session_bindings_module=session_bindings_module,
        session_keys_module=modules.session_keys_module,
        normalize_partition_fn=modules.normalize_telegram_session_partition_mode,
        telegram_prefix=telegram_prefix,
        discord_prefix=discord_prefix,
    )
    namespace["normalize_session_partition"] = session_helpers["normalize_session_partition"]
    namespace["expected_session_keys"] = session_helpers["expected_session_keys"]
    namespace["expected_session_key"] = session_helpers["expected_session_key"]
    namespace["expected_session_scope_values"] = session_helpers["expected_session_scope_values"]
    namespace["expected_session_scope_prefixes"] = session_helpers[
        "expected_session_scope_prefixes"
    ]
    namespace["expected_recipient_key"] = session_helpers["expected_recipient_key"]
    namespace["post_webhook_update"] = modules.http_module.post_webhook_update
