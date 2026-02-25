#!/usr/bin/env python3
"""Log monitoring loop for Discord ACL runtime probes."""

from __future__ import annotations

import sys
import time
from typing import Any


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
    last_log_activity = monotonic_fn()
    seen_dispatch = False
    seen_bot = False
    bot_lines: list[str] = []
    command_reply_observations: list[dict[str, object]] = []
    json_reply_summary_observations: list[dict[str, str]] = []
    matched_expect_event = False
    matched_expect_reply_json = [False] * len(expect_fields)
    forbid_log_patterns = compile_patterns_fn((forbidden_log_pattern,))

    while True:
        if monotonic_fn() > deadline:
            break

        cursor, chunk = blackbox.read_new_lines(config.log_file, cursor)
        if chunk:
            last_log_activity = monotonic_fn()
            normalized_chunk = [blackbox.strip_ansi(line) for line in chunk]
            if not config.no_follow:
                for line in chunk:
                    print(f"[log] {line}")
            for line in normalized_chunk:
                event = blackbox.extract_event_token(line)
                if event == case.event_name:
                    tokens = blackbox.parse_log_tokens(line)
                    recipient = tokens.get("recipient", "")
                    if recipient == expected_recipient:
                        matched_expect_event = True

                reply_observation = blackbox.parse_command_reply_event_line(line)
                if reply_observation:
                    command_reply_observations.append(reply_observation)

                summary_observation = blackbox.parse_command_reply_json_summary_line(line)
                if summary_observation:
                    json_reply_summary_observations.append(summary_observation)
                    if (
                        summary_observation.get("event") == case.event_name
                        and summary_observation.get("recipient") == expected_recipient
                    ):
                        for idx, (key, expected) in enumerate(expect_fields):
                            if matched_expect_reply_json[idx]:
                                continue
                            if reply_json_field_matches_fn(
                                key=key,
                                expected=expected,
                                observation=summary_observation,
                                expected_session_scopes_values=expected_session_scopes_values,
                                target_session_scope_placeholder=target_session_scope_placeholder,
                            ):
                                matched_expect_reply_json[idx] = True

                for pattern in forbid_log_patterns:
                    if pattern.search(line):
                        print(
                            f"[{case.case_id}] forbidden log matched: {pattern.pattern}",
                            file=sys.stderr,
                        )
                        print(f"  line={line}", file=sys.stderr)
                        return 5, {}
                if any(pattern in line for pattern in error_patterns):
                    print(f"[{case.case_id}] fail-fast error log detected.", file=sys.stderr)
                    print(f"  line={line}", file=sys.stderr)
                    return 6, {}

                if "discord ingress parsed message" in line:
                    seen_dispatch = True
                if "→ Bot:" in line:
                    seen_bot = True
                    bot_lines.append(line)

            if seen_dispatch and matched_expect_event and all(matched_expect_reply_json):
                break

        if config.max_idle_secs > 0 and (monotonic_fn() - last_log_activity) > config.max_idle_secs:
            print(f"[{case.case_id}] max-idle exceeded.", file=sys.stderr)
            return 7, {}
        sleep_fn(1)

    if not seen_dispatch:
        print(f"[{case.case_id}] timed out: no discord ingress dispatch marker.", file=sys.stderr)
        return 9, {}
    if not matched_expect_event or not all(matched_expect_reply_json):
        print(f"[{case.case_id}] timed out: missing expected event/json fields.", file=sys.stderr)
        print(f"  expect_event={case.event_name}", file=sys.stderr)
        if expect_fields:
            print(
                "  expect_reply_json=" + ",".join(f"{key}={value}" for key, value in expect_fields),
                file=sys.stderr,
            )
        return 8, {}

    return 0, {
        "seen_bot": seen_bot,
        "bot_lines": bot_lines,
        "command_reply_observations": command_reply_observations,
        "json_reply_summary_observations": json_reply_summary_observations,
    }
