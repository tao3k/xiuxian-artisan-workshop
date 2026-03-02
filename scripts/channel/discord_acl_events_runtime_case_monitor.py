#!/usr/bin/env python3
"""Log monitoring loop for Discord ACL runtime probes."""

from __future__ import annotations

import sys
import time
from typing import Any

from discord_acl_events_runtime_case_monitor_chunk import process_chunk
from discord_acl_events_runtime_case_monitor_result import finalize_monitor
from discord_acl_events_runtime_case_monitor_state import new_monitor_state


def monitor_case_until_completion(
    *,
    config: Any,
    case: Any,
    blackbox: Any,
    expect_fields: tuple[tuple[str, str], ...],
    expected_recipient: str,
    expected_session_scopes_values: tuple[str, ...],
    compile_patterns_fn: Any,
    reply_json_field_matches_fn: Any,
    forbidden_log_pattern: str,
    error_patterns: tuple[str, ...],
    target_session_scope_placeholder: str,
    monotonic_fn: Any = time.monotonic,
    sleep_fn: Any = time.sleep,
) -> tuple[int, dict[str, Any]]:
    """Follow runtime logs until case completion, timeout, or fail-fast condition."""
    cursor = blackbox.count_lines(config.log_file)
    deadline = monotonic_fn() + config.max_wait_secs
    state = new_monitor_state(expect_fields_count=len(expect_fields), now=monotonic_fn())
    forbid_log_patterns = tuple(compile_patterns_fn((forbidden_log_pattern,)))

    while True:
        if monotonic_fn() > deadline:
            break

        cursor, chunk = blackbox.read_new_lines(config.log_file, cursor)
        if chunk:
            state.last_log_activity = monotonic_fn()
            fail_code = process_chunk(
                chunk=chunk,
                state=state,
                config=config,
                case=case,
                blackbox=blackbox,
                expect_fields=expect_fields,
                expected_recipient=expected_recipient,
                expected_session_scopes_values=expected_session_scopes_values,
                reply_json_field_matches_fn=reply_json_field_matches_fn,
                forbid_log_patterns=forbid_log_patterns,
                error_patterns=error_patterns,
                target_session_scope_placeholder=target_session_scope_placeholder,
            )
            if fail_code is not None:
                return fail_code, {}
            if (
                state.seen_dispatch
                and state.matched_expect_event
                and all(state.matched_expect_reply_json)
            ):
                break

        if (
            config.max_idle_secs > 0
            and (monotonic_fn() - state.last_log_activity) > config.max_idle_secs
        ):
            print(f"[{case.case_id}] max-idle exceeded.", file=sys.stderr)
            return 7, {}
        sleep_fn(1)

    return finalize_monitor(state=state, case=case, expect_fields=expect_fields)
