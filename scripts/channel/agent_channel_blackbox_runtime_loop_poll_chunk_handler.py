#!/usr/bin/env python3
"""Chunk-processing helpers for blackbox runtime probe polling."""

from __future__ import annotations

from typing import Any

from agent_channel_blackbox_runtime_loop_poll_chunk import process_normalized_chunk
from agent_channel_blackbox_runtime_loop_poll_model import ProbeLoopOutcome, outcome_from_state


def handle_polled_chunk(
    cfg: Any,
    *,
    runtime_state: Any,
    loop_state: Any,
    update_id: int,
    trace_mode: bool,
    trace_id: str,
    chunk: list[str],
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
    helpers_module: Any,
    monotonic_fn: Any,
) -> tuple[ProbeLoopOutcome | None, bool]:
    """Handle one read chunk and return optional final outcome or break flag."""
    normalized_chunk = [strip_ansi_fn(line) for line in chunk]
    loop_state.last_log_activity = monotonic_fn()
    if cfg.follow_logs:
        for line in chunk:
            print(f"[log] {line}")

    exit_code, allow_no_bot_success = process_normalized_chunk(
        cfg,
        runtime_state=runtime_state,
        loop_state=loop_state,
        update_id=update_id,
        trace_mode=trace_mode,
        trace_id=trace_id,
        normalized_chunk=normalized_chunk,
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
        helpers_module=helpers_module,
        monotonic_fn=monotonic_fn,
    )
    if exit_code is not None:
        return (
            outcome_from_state(
                loop_state,
                trace_mode=trace_mode,
                exit_code=exit_code,
                allow_no_bot_success=False,
            ),
            False,
        )
    if allow_no_bot_success:
        return (
            outcome_from_state(
                loop_state,
                trace_mode=trace_mode,
                exit_code=None,
                allow_no_bot_success=True,
            ),
            False,
        )
    return None, bool(loop_state.dispatch_session_mismatch_line)
