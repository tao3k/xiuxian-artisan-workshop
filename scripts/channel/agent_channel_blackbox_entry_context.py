#!/usr/bin/env python3
"""Context-driven entry helpers for agent channel blackbox probe."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class BlackboxEntryContext:
    """Dependency bundle for blackbox parse/build/run entry flow."""

    entry_flow_module: Any
    entry_bindings_module: Any
    config_module: Any
    runtime_module: Any
    default_telegram_webhook_url_fn: Any
    target_session_scope_placeholder: str
    probe_config_cls: Any
    session_ids_from_runtime_log_fn: Any
    username_from_settings_fn: Any
    username_from_runtime_log_fn: Any
    parse_expected_field_fn: Any
    parse_allow_chat_ids_fn: Any
    normalize_session_partition_fn: Any
    telegram_webhook_secret_token_fn: Any
    count_lines_fn: Any
    next_update_id_fn: Any
    build_probe_message_fn: Any
    build_update_payload_fn: Any
    post_webhook_update_fn: Any
    expected_session_keys_fn: Any
    expected_session_scope_values_fn: Any
    expected_session_scope_prefixes_fn: Any
    expected_session_key_fn: Any
    expected_recipient_key_fn: Any
    read_new_lines_fn: Any
    tail_lines_fn: Any
    strip_ansi_fn: Any
    extract_event_token_fn: Any
    extract_session_key_token_fn: Any
    parse_command_reply_event_line_fn: Any
    parse_command_reply_json_summary_line_fn: Any
    telegram_send_retry_grace_seconds_fn: Any
    parse_log_tokens_fn: Any
    error_patterns: tuple[str, ...]
    mcp_observability_events: tuple[str, ...]
    mcp_waiting_events: frozenset[str]


def parse_args(context: BlackboxEntryContext) -> Any:
    """Parse blackbox entrypoint arguments."""
    return context.entry_flow_module.parse_args(
        entry_bindings_module=context.entry_bindings_module,
        config_module=context.config_module,
        default_telegram_webhook_url_fn=context.default_telegram_webhook_url_fn,
        target_session_scope_placeholder=context.target_session_scope_placeholder,
    )


def build_config(args: Any, context: BlackboxEntryContext) -> Any:
    """Build probe config from parsed args."""
    return context.entry_flow_module.build_config(
        args,
        entry_bindings_module=context.entry_bindings_module,
        config_module=context.config_module,
        probe_config_cls=context.probe_config_cls,
        session_ids_from_runtime_log_fn=context.session_ids_from_runtime_log_fn,
        username_from_settings_fn=context.username_from_settings_fn,
        username_from_runtime_log_fn=context.username_from_runtime_log_fn,
        parse_expected_field_fn=context.parse_expected_field_fn,
        parse_allow_chat_ids_fn=context.parse_allow_chat_ids_fn,
        normalize_session_partition_fn=context.normalize_session_partition_fn,
        telegram_webhook_secret_token_fn=context.telegram_webhook_secret_token_fn,
    )


def run_probe(cfg: Any, context: BlackboxEntryContext) -> int:
    """Run blackbox probe from typed config."""
    return context.entry_flow_module.run_probe(
        cfg,
        entry_bindings_module=context.entry_bindings_module,
        runtime_module=context.runtime_module,
        count_lines_fn=context.count_lines_fn,
        next_update_id_fn=context.next_update_id_fn,
        build_probe_message_fn=context.build_probe_message_fn,
        build_update_payload_fn=context.build_update_payload_fn,
        post_webhook_update_fn=context.post_webhook_update_fn,
        expected_session_keys_fn=context.expected_session_keys_fn,
        expected_session_scope_values_fn=context.expected_session_scope_values_fn,
        expected_session_scope_prefixes_fn=context.expected_session_scope_prefixes_fn,
        expected_session_key_fn=context.expected_session_key_fn,
        expected_recipient_key_fn=context.expected_recipient_key_fn,
        read_new_lines_fn=context.read_new_lines_fn,
        tail_lines_fn=context.tail_lines_fn,
        strip_ansi_fn=context.strip_ansi_fn,
        extract_event_token_fn=context.extract_event_token_fn,
        extract_session_key_token_fn=context.extract_session_key_token_fn,
        parse_command_reply_event_line_fn=context.parse_command_reply_event_line_fn,
        parse_command_reply_json_summary_line_fn=context.parse_command_reply_json_summary_line_fn,
        telegram_send_retry_grace_seconds_fn=context.telegram_send_retry_grace_seconds_fn,
        parse_log_tokens_fn=context.parse_log_tokens_fn,
        error_patterns=context.error_patterns,
        mcp_observability_events=context.mcp_observability_events,
        mcp_waiting_events=context.mcp_waiting_events,
        target_session_scope_placeholder=context.target_session_scope_placeholder,
    )
