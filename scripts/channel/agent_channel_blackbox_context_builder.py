#!/usr/bin/env python3
"""Build dynamic entry context snapshots for blackbox probe module."""

from __future__ import annotations

from typing import Any

from agent_channel_blackbox_entry_context import BlackboxEntryContext


def build_entry_context(
    *,
    namespace: dict[str, Any],
    modules: Any,
    entry_flow_module: Any,
    entry_bindings_module: Any,
    probe_config_cls: Any,
    target_session_scope_placeholder: str,
) -> BlackboxEntryContext:
    """Build context from current module namespace to keep monkeypatch seams live."""
    return BlackboxEntryContext(
        entry_flow_module=entry_flow_module,
        entry_bindings_module=entry_bindings_module,
        config_module=modules.config_module,
        runtime_module=modules.runtime_module,
        default_telegram_webhook_url_fn=modules.default_telegram_webhook_url,
        target_session_scope_placeholder=target_session_scope_placeholder,
        probe_config_cls=probe_config_cls,
        session_ids_from_runtime_log_fn=modules.session_ids_from_runtime_log,
        username_from_settings_fn=namespace["username_from_settings"],
        username_from_runtime_log_fn=namespace["infer_username_from_log"],
        parse_expected_field_fn=namespace["parse_expected_field"],
        parse_allow_chat_ids_fn=namespace["parse_allow_chat_ids"],
        normalize_session_partition_fn=namespace["normalize_session_partition"],
        telegram_webhook_secret_token_fn=modules.telegram_webhook_secret_token,
        count_lines_fn=namespace["count_lines"],
        next_update_id_fn=namespace["next_update_id"],
        build_probe_message_fn=namespace["build_probe_message"],
        build_update_payload_fn=namespace["build_update_payload"],
        post_webhook_update_fn=namespace["post_webhook_update"],
        expected_session_keys_fn=namespace["expected_session_keys"],
        expected_session_scope_values_fn=namespace["expected_session_scope_values"],
        expected_session_scope_prefixes_fn=namespace["expected_session_scope_prefixes"],
        expected_session_key_fn=namespace["expected_session_key"],
        expected_recipient_key_fn=namespace["expected_recipient_key"],
        read_new_lines_fn=namespace["read_new_lines"],
        tail_lines_fn=namespace["tail_lines"],
        strip_ansi_fn=namespace["strip_ansi"],
        extract_event_token_fn=namespace["extract_event_token"],
        extract_session_key_token_fn=namespace["extract_session_key_token"],
        parse_command_reply_event_line_fn=namespace["parse_command_reply_event_line"],
        parse_command_reply_json_summary_line_fn=namespace["parse_command_reply_json_summary_line"],
        telegram_send_retry_grace_seconds_fn=namespace["telegram_send_retry_grace_seconds"],
        parse_log_tokens_fn=namespace["parse_log_tokens"],
        error_patterns=namespace["ERROR_PATTERNS"],
        mcp_observability_events=namespace["MCP_OBSERVABILITY_EVENTS"],
        mcp_waiting_events=namespace["MCP_WAITING_EVENTS"],
    )
