#!/usr/bin/env python3
"""Polling loop implementation for agent channel blackbox runtime probe."""

from __future__ import annotations

import sys
import time
from typing import Any

from agent_channel_blackbox_runtime_loop_poll_chunk import process_normalized_chunk
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
            normalized_chunk = [strip_ansi_fn(line) for line in chunk]
            loop_state.last_log_activity = monotonic_fn()
            if cfg.follow_logs:
                for line in chunk:
                    print(f"[log] {line}")

            exit_code, allow_no_bot_success = process_normalized_chunk(
                cfg,
                runtime_state=state,
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
                return outcome_from_state(
                    loop_state,
                    trace_mode=trace_mode,
                    exit_code=exit_code,
                    allow_no_bot_success=False,
                )
            if allow_no_bot_success:
                return outcome_from_state(
                    loop_state,
                    trace_mode=trace_mode,
                    exit_code=None,
                    allow_no_bot_success=True,
                )
            if loop_state.dispatch_session_mismatch_line:
                break

        if loop_state.seen_bot and helpers_module.all_expectations_satisfied(state):
            break

        if (
            cfg.max_idle_secs is not None
            and (monotonic_fn() - loop_state.last_log_activity) > cfg.max_idle_secs
        ):
            if loop_state.retry_grace_until and monotonic_fn() <= loop_state.retry_grace_until:
                sleep_fn(0.2)
                continue
            print("", file=sys.stderr)
            print("Probe failed: max-idle exceeded with no new logs.", file=sys.stderr)
            print(f"  max_idle_secs={cfg.max_idle_secs}", file=sys.stderr)
            return outcome_from_state(
                loop_state,
                trace_mode=trace_mode,
                exit_code=7,
                allow_no_bot_success=False,
            )

        sleep_fn(1)

    return outcome_from_state(
        loop_state,
        trace_mode=trace_mode,
        exit_code=None,
        allow_no_bot_success=False,
    )
