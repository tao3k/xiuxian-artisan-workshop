#!/usr/bin/env python3
"""Runtime probe loop for agent channel blackbox."""

from __future__ import annotations

import importlib
import os
from typing import Any

_helpers_module = importlib.import_module("agent_channel_blackbox_runtime_helpers")
_outcome_module = importlib.import_module("agent_channel_blackbox_runtime_outcome")
_poll_module = importlib.import_module("agent_channel_blackbox_runtime_loop_poll")
_http_loop_module = importlib.import_module("agent_channel_blackbox_runtime_loop_http")


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
    cfg.log_file.parent.mkdir(parents=True, exist_ok=True)
    cursor = count_lines_fn(cfg.log_file)

    update_id = next_update_id_fn(cfg.strong_update_id)
    trace_id = f"bbx-{update_id}-{os.getpid()}"
    message_text = build_probe_message_fn(cfg.prompt, trace_id)

    post_error = _http_loop_module.handle_webhook_post(
        cfg,
        update_id=update_id,
        message_text=message_text,
        build_update_payload_fn=build_update_payload_fn,
        post_webhook_update_fn=post_webhook_update_fn,
    )
    if post_error is not None:
        return post_error

    _helpers_module.print_probe_intro(
        cfg,
        update_id=update_id,
        trace_id=trace_id,
        message_text=message_text,
    )
    state = _helpers_module.build_probe_runtime_state(
        cfg,
        expected_session_keys_fn=expected_session_keys_fn,
        expected_session_scope_values_fn=expected_session_scope_values_fn,
        expected_session_scope_prefixes_fn=expected_session_scope_prefixes_fn,
        expected_session_key_fn=expected_session_key_fn,
        expected_recipient_key_fn=expected_recipient_key_fn,
    )

    def finish(code: int) -> int:
        _helpers_module.emit_mcp_diagnostics(
            state,
            mcp_observability_events=mcp_observability_events,
        )
        return code

    trace_mode = trace_id in message_text
    loop_outcome = _poll_module.poll_probe_logs(
        cfg,
        state=state,
        cursor=cursor,
        update_id=update_id,
        trace_mode=trace_mode,
        trace_id=trace_id,
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

    if loop_outcome.exit_code is not None:
        return finish(loop_outcome.exit_code)
    if loop_outcome.allow_no_bot_success:
        return finish(0)

    return _outcome_module.handle_post_loop_outcome(
        cfg=cfg,
        state=state,
        finish_fn=finish,
        tail_lines_fn=tail_lines_fn,
        helpers_module=_helpers_module,
        trace_mode=loop_outcome.trace_mode,
        seen_trace=loop_outcome.seen_trace,
        seen_user_dispatch=loop_outcome.seen_user_dispatch,
        seen_bot=loop_outcome.seen_bot,
        bot_line=loop_outcome.bot_line,
        error_line=loop_outcome.error_line,
        dedup_duplicate_line=loop_outcome.dedup_duplicate_line,
        dispatch_session_mismatch_line=loop_outcome.dispatch_session_mismatch_line,
        webhook_seen=loop_outcome.webhook_seen,
        trace_id=trace_id,
    )
