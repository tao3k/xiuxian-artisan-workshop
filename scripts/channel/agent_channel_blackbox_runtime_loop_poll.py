#!/usr/bin/env python3
"""Polling loop implementation for agent channel blackbox runtime probe."""

from __future__ import annotations

import time
from typing import Any

from agent_channel_blackbox_runtime_loop_poll_chunk_handler import handle_polled_chunk
from agent_channel_blackbox_runtime_loop_poll_idle import handle_idle_timeout
from agent_channel_blackbox_runtime_loop_poll_model import (
    ProbeLoopOutcome,
    build_initial_state,
    outcome_from_state,
)


def poll_probe_logs(
    cfg: Any,
    *,
    state: Any,
    cursor: int,
    update_id: int,
    trace_mode: bool,
    trace_id: str,
    read_new_lines_fn: Any,
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
    monotonic_fn: Any = time.monotonic,
    sleep_fn: Any = time.sleep,
) -> ProbeLoopOutcome:
    """Poll runtime logs until probe completion, timeout, or fail-fast condition."""
    loop_state = build_initial_state(now_monotonic=monotonic_fn())
    deadline = monotonic_fn() + cfg.max_wait_secs if cfg.max_wait_secs is not None else None

    while True:
        if deadline is not None and monotonic_fn() > deadline:
            break

        cursor, chunk = read_new_lines_fn(cfg.log_file, cursor)
        if chunk:
            outcome, should_break = handle_polled_chunk(
                cfg,
                runtime_state=state,
                loop_state=loop_state,
                update_id=update_id,
                trace_mode=trace_mode,
                trace_id=trace_id,
                chunk=chunk,
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
                helpers_module=helpers_module,
                monotonic_fn=monotonic_fn,
            )
            if outcome is not None:
                return outcome
            if should_break:
                break

        if loop_state.seen_bot and helpers_module.all_expectations_satisfied(state):
            break

        idle_outcome, skipped_default_sleep = handle_idle_timeout(
            cfg,
            loop_state=loop_state,
            trace_mode=trace_mode,
            monotonic_fn=monotonic_fn,
            sleep_fn=sleep_fn,
        )
        if idle_outcome is not None:
            return idle_outcome
        if skipped_default_sleep:
            continue

        sleep_fn(1)

    return outcome_from_state(
        loop_state,
        trace_mode=trace_mode,
        exit_code=None,
        allow_no_bot_success=False,
    )
