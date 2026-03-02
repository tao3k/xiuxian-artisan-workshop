#!/usr/bin/env python3
"""Runtime delegations for channel blackbox entry flow wrappers."""

from __future__ import annotations

from typing import Any


def run_probe(
    cfg: Any,
    *,
    entry_bindings_module: Any,
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
    """Run one probe execution through runtime bindings."""
    return entry_bindings_module.run_probe(
        cfg,
        runtime_module=runtime_module,
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
    entry_bindings_module: Any,
    parse_args_fn: Any,
    build_config_fn: Any,
    run_probe_fn: Any,
) -> int:
    """Run blackbox top-level command."""
    return entry_bindings_module.run_main(
        parse_args_fn=parse_args_fn,
        build_config_fn=build_config_fn,
        run_probe_fn=run_probe_fn,
    )
