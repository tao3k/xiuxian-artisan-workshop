#!/usr/bin/env python3
"""Chunk processing helpers for agent blackbox runtime polling."""

from __future__ import annotations

import sys
from typing import Any


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
                return 5, False

    for line in normalized_chunk:
        if f"update_id=Some({update_id})" in line:
            loop_state.webhook_seen = True
        if str(update_id) in line and "duplicate update" in line.lower():
            loop_state.dedup_duplicate_line = line
        if trace_mode and trace_id in line:
            observed_session_key = extract_session_key_token_fn(line)
            if observed_session_key and observed_session_key not in runtime_state.expected_sessions:
                loop_state.dispatch_session_mismatch_line = line
                break

    if (
        trace_mode
        and not loop_state.seen_trace
        and any(trace_id in line for line in normalized_chunk)
    ):
        loop_state.seen_trace = True

    if not loop_state.seen_user_dispatch:
        if trace_mode:
            loop_state.seen_user_dispatch = any(
                "← User:" in line and trace_id in line for line in normalized_chunk
            )
        else:
            loop_state.seen_user_dispatch = any(
                ("← User:" in line or "Parsed message, forwarding to agent" in line)
                and cfg.prompt in line
                for line in normalized_chunk
            )

    if loop_state.seen_user_dispatch:
        for line in normalized_chunk:
            if any(pattern in line for pattern in error_patterns):
                loop_state.error_line = line
                if cfg.fail_fast_error_logs:
                    print("", file=sys.stderr)
                    print("Probe failed: fail-fast error log detected.", file=sys.stderr)
                    print(f"  line={line}", file=sys.stderr)
                    return 6, False

        for line in normalized_chunk:
            if "→ Bot:" in line:
                loop_state.seen_bot = True
                loop_state.bot_line = line
                helpers_module.mark_expect_bot_patterns(runtime_state, line)

    if (
        cfg.allow_no_bot
        and loop_state.seen_user_dispatch
        and helpers_module.all_expectations_satisfied(runtime_state)
    ):
        session_ok, mismatch_context = helpers_module.validate_target_session_scope(
            cfg, runtime_state
        )
        if not session_ok:
            print("Probe failed: command reply session_key mismatch.", file=sys.stderr)
            print(
                f"  expected_session_keys={list(runtime_state.expected_sessions)}", file=sys.stderr
            )
            print(f"  {mismatch_context}", file=sys.stderr)
            return 10, False
        print("")
        print("Blackbox probe succeeded (allow-no-bot mode).")
        print(
            "All expect-event / expect-reply-json-field / expect-log / expect-bot checks are satisfied."
        )
        return None, True

    return None, False
