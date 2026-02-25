#!/usr/bin/env python3
"""Entrypoint bindings for channel blackbox probe."""

from __future__ import annotations

import sys
from typing import Any


def parse_args(
    *,
    config_module: Any,
    default_telegram_webhook_url_fn: Any,
    target_session_scope_placeholder: str,
) -> Any:
    """Parse CLI args for channel blackbox probe."""
    return config_module.parse_args(
        default_telegram_webhook_url_fn=default_telegram_webhook_url_fn,
        target_session_scope_placeholder=target_session_scope_placeholder,
    )


def build_config(
    args: Any,
    *,
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
    """Build normalized probe config."""
    return config_module.build_config(
        args,
        probe_config_cls=probe_config_cls,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
        parse_expected_field_fn=parse_expected_field_fn,
        parse_allow_chat_ids_fn=parse_allow_chat_ids_fn,
        normalize_session_partition_fn=normalize_session_partition_fn,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token_fn,
    )


def run_probe(
    cfg: Any,
    *,
    runtime_module: Any,
    count_lines_fn: Any,
    next_update_id_fn: Any,
    build_probe_message_fn: Any,
    build_update_payload_fn: Any,
    post_webhook_update_fn: Any,
    expected_session_keys_fn: Any,
    expected_session_scope_values_fn: Any,
    expected_session_scope_prefixes_fn: Any,
    expected_session_key_fn: Any,
    expected_recipient_key_fn: Any,
    read_new_lines_fn: Any,
    tail_lines_fn: Any,
    strip_ansi_fn: Any,
    extract_event_token_fn: Any,
    extract_session_key_token_fn: Any,
    parse_command_reply_event_line_fn: Any,
    parse_command_reply_json_summary_line_fn: Any,
    telegram_send_retry_grace_seconds_fn: Any,
    parse_log_tokens_fn: Any,
    error_patterns: tuple[str, ...],
    mcp_observability_events: tuple[str, ...],
    mcp_waiting_events: frozenset[str],
    target_session_scope_placeholder: str,
) -> int:
    """Run one blackbox probe end-to-end."""
    return runtime_module.run_probe(
        cfg,
        count_lines_fn=count_lines_fn,
        next_update_id_fn=next_update_id_fn,
        build_probe_message_fn=build_probe_message_fn,
        build_update_payload_fn=build_update_payload_fn,
        post_webhook_update_fn=post_webhook_update_fn,
        expected_session_keys_fn=expected_session_keys_fn,
        expected_session_scope_values_fn=expected_session_scope_values_fn,
        expected_session_scope_prefixes_fn=expected_session_scope_prefixes_fn,
        expected_session_key_fn=expected_session_key_fn,
        expected_recipient_key_fn=expected_recipient_key_fn,
        read_new_lines_fn=read_new_lines_fn,
        tail_lines_fn=tail_lines_fn,
        strip_ansi_fn=strip_ansi_fn,
        extract_event_token_fn=extract_event_token_fn,
        extract_session_key_token_fn=extract_session_key_token_fn,
        parse_command_reply_event_line_fn=parse_command_reply_event_line_fn,
        parse_command_reply_json_summary_line_fn=parse_command_reply_json_summary_line_fn,
        telegram_send_retry_grace_seconds_fn=telegram_send_retry_grace_seconds_fn,
        parse_log_tokens_fn=parse_log_tokens_fn,
        error_patterns=error_patterns,
        mcp_observability_events=mcp_observability_events,
        mcp_waiting_events=mcp_waiting_events,
        target_session_scope_placeholder=target_session_scope_placeholder,
    )


def run_main(
    *,
    parse_args_fn: Any,
    build_config_fn: Any,
    run_probe_fn: Any,
) -> int:
    """Run top-level blackbox command."""
    args = parse_args_fn()
    try:
        config = build_config_fn(args)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1
    try:
        return run_probe_fn(config)
    except KeyboardInterrupt:
        print("\nProbe interrupted by user.", file=sys.stderr)
        return 130
