#!/usr/bin/env python3
"""Event observation processing for blackbox probe log chunks."""

from __future__ import annotations

import sys
from typing import Any


def process_event_lines(
    cfg: Any,
    runtime_state: Any,
    loop_state: Any,
    *,
    normalized_chunk: list[str],
    extract_event_token_fn: Any,
    parse_command_reply_event_line_fn: Any,
    parse_command_reply_json_summary_line_fn: Any,
    telegram_send_retry_grace_seconds_fn: Any,
    parse_log_tokens_fn: Any,
    mcp_observability_events: tuple[str, ...],
    mcp_waiting_events: frozenset[str],
    target_session_scope_placeholder: str,
    helpers_module: Any,
    monotonic_fn: Any,
) -> int | None:
    """Process event-level observations in one normalized chunk."""
    for line in normalized_chunk:
        event_token = extract_event_token_fn(line)
        if event_token:
            helpers_module.mark_event_expectation(
                cfg,
                runtime_state,
                event_token=event_token,
                line=line,
                parse_log_tokens_fn=parse_log_tokens_fn,
            )
            helpers_module.record_mcp_event(
                runtime_state,
                event_token=event_token,
                mcp_observability_events=mcp_observability_events,
                mcp_waiting_events=mcp_waiting_events,
            )

        reply_obs = parse_command_reply_event_line_fn(line)
        if reply_obs:
            runtime_state.command_reply_observations.append(reply_obs)

        json_summary_obs = parse_command_reply_json_summary_line_fn(line)
        if json_summary_obs:
            runtime_state.json_reply_summary_observations.append(json_summary_obs)
            helpers_module.mark_reply_json_expectation(
                cfg,
                runtime_state,
                json_summary_obs=json_summary_obs,
                target_session_scope_placeholder=target_session_scope_placeholder,
            )

        retry_grace_secs = telegram_send_retry_grace_seconds_fn(line)
        if retry_grace_secs is not None:
            grace_until = monotonic_fn() + retry_grace_secs + 2.0
            if grace_until > loop_state.retry_grace_until:
                loop_state.retry_grace_until = grace_until

        helpers_module.mark_expect_log_patterns(runtime_state, line)

        for pattern in runtime_state.forbid_log_compiled:
            if pattern.search(line):
                print("", file=sys.stderr)
                print("Probe failed: forbidden log regex matched.", file=sys.stderr)
                print(f"  regex={pattern.pattern}", file=sys.stderr)
                print(f"  line={line}", file=sys.stderr)
                return 5
    return None
