#!/usr/bin/env python3
"""Chunk processing helpers for agent blackbox runtime polling."""

from __future__ import annotations

import sys
from typing import Any

from agent_channel_blackbox_runtime_loop_poll_chunk_events import process_event_lines
from agent_channel_blackbox_runtime_loop_poll_chunk_observe import process_state_lines


def _allow_no_bot_success(
    cfg: Any,
    runtime_state: Any,
    *,
    loop_state: Any,
    helpers_module: Any,
) -> tuple[int | None, bool]:
    if not (
        cfg.allow_no_bot
        and loop_state.seen_user_dispatch
        and helpers_module.all_expectations_satisfied(runtime_state)
    ):
        return None, False

    session_ok, mismatch_context = helpers_module.validate_target_session_scope(
        cfg,
        runtime_state,
    )
    if not session_ok:
        print("Probe failed: command reply session_key mismatch.", file=sys.stderr)
        print(
            f"  expected_session_keys={list(runtime_state.expected_sessions)}",
            file=sys.stderr,
        )
        print(f"  {mismatch_context}", file=sys.stderr)
        return 10, False

    print("")
    print("Blackbox probe succeeded (allow-no-bot mode).")
    print(
        "All expect-event / expect-reply-json-field / expect-log / expect-bot checks are satisfied."
    )
    return None, True


def process_normalized_chunk(
    cfg: Any,
    runtime_state: Any,
    loop_state: Any,
    *,
    update_id: int,
    trace_mode: bool,
    trace_id: str,
    normalized_chunk: list[str],
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
) -> tuple[int | None, bool]:
    """Process one normalized log chunk and return (exit_code, allow_no_bot_success)."""
    event_exit_code = process_event_lines(
        cfg,
        runtime_state,
        loop_state,
        normalized_chunk=normalized_chunk,
        extract_event_token_fn=extract_event_token_fn,
        parse_command_reply_event_line_fn=parse_command_reply_event_line_fn,
        parse_command_reply_json_summary_line_fn=parse_command_reply_json_summary_line_fn,
        telegram_send_retry_grace_seconds_fn=telegram_send_retry_grace_seconds_fn,
        parse_log_tokens_fn=parse_log_tokens_fn,
        mcp_observability_events=mcp_observability_events,
        mcp_waiting_events=mcp_waiting_events,
        target_session_scope_placeholder=target_session_scope_placeholder,
        helpers_module=helpers_module,
        monotonic_fn=monotonic_fn,
    )
    if event_exit_code is not None:
        return event_exit_code, False

    state_exit_code = process_state_lines(
        cfg,
        runtime_state,
        loop_state,
        update_id=update_id,
        trace_mode=trace_mode,
        trace_id=trace_id,
        normalized_chunk=normalized_chunk,
        extract_session_key_token_fn=extract_session_key_token_fn,
        error_patterns=error_patterns,
        helpers_module=helpers_module,
    )
    if state_exit_code is not None:
        return state_exit_code, False

    return _allow_no_bot_success(
        cfg,
        runtime_state,
        loop_state=loop_state,
        helpers_module=helpers_module,
    )
