#!/usr/bin/env python3
"""Runtime probe loop for agent channel blackbox."""

from __future__ import annotations

import importlib
from typing import Any

_helpers_module = importlib.import_module("agent_channel_blackbox_runtime_helpers")
_outcome_module = importlib.import_module("agent_channel_blackbox_runtime_outcome")
_poll_module = importlib.import_module("agent_channel_blackbox_runtime_loop_poll")
_http_loop_module = importlib.import_module("agent_channel_blackbox_runtime_loop_http")
_prepare_module = importlib.import_module("agent_channel_blackbox_runtime_loop_prepare")
_finalize_module = importlib.import_module("agent_channel_blackbox_runtime_loop_finalize")


def run_probe(
    cfg: Any,
    *,
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
    prepared, prepare_error = _prepare_module.prepare_probe(
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
        helpers_module=_helpers_module,
        http_loop_module=_http_loop_module,
    )
    if prepare_error is not None:
        return prepare_error
    assert prepared is not None
    state = prepared.state

    def finish(code: int) -> int:
        _helpers_module.emit_mcp_diagnostics(
            state,
            mcp_observability_events=mcp_observability_events,
        )
        return code

    loop_outcome = _poll_module.poll_probe_logs(
        cfg,
        state=state,
        cursor=prepared.cursor,
        update_id=prepared.update_id,
        trace_mode=prepared.trace_mode,
        trace_id=prepared.trace_id,
        read_new_lines_fn=read_new_lines_fn,
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
        helpers_module=_helpers_module,
    )

    return _finalize_module.finalize_probe_outcome(
        cfg=cfg,
        state=state,
        loop_outcome=loop_outcome,
        trace_id=prepared.trace_id,
        finish_fn=finish,
        tail_lines_fn=tail_lines_fn,
        helpers_module=_helpers_module,
        outcome_module=_outcome_module,
    )
