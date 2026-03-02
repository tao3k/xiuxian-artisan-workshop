#!/usr/bin/env python3
"""Chunk-processing helpers for Discord ACL runtime case monitoring."""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from discord_acl_events_runtime_case_monitor_state import MonitorState


def process_chunk(
    *,
    chunk: list[str],
    state: MonitorState,
    config: Any,
    case: Any,
    blackbox: Any,
    expect_fields: tuple[tuple[str, str], ...],
    expected_recipient: str,
    expected_session_scopes_values: tuple[str, ...],
    reply_json_field_matches_fn: Any,
    forbid_log_patterns: tuple[Any, ...],
    error_patterns: tuple[str, ...],
    target_session_scope_placeholder: str,
) -> int | None:
    """Process one log chunk and update monitor state, optionally fail-fast."""
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
                state.matched_expect_event = True

        reply_observation = blackbox.parse_command_reply_event_line(line)
        if reply_observation:
            state.command_reply_observations.append(reply_observation)

        summary_observation = blackbox.parse_command_reply_json_summary_line(line)
        if summary_observation:
            state.json_reply_summary_observations.append(summary_observation)
            if (
                summary_observation.get("event") == case.event_name
                and summary_observation.get("recipient") == expected_recipient
            ):
                for idx, (key, expected) in enumerate(expect_fields):
                    if state.matched_expect_reply_json[idx]:
                        continue
                    if reply_json_field_matches_fn(
                        key=key,
                        expected=expected,
                        observation=summary_observation,
                        expected_session_scopes_values=expected_session_scopes_values,
                        target_session_scope_placeholder=target_session_scope_placeholder,
                    ):
                        state.matched_expect_reply_json[idx] = True

        for pattern in forbid_log_patterns:
            if pattern.search(line):
                print(
                    f"[{case.case_id}] forbidden log matched: {pattern.pattern}",
                    file=sys.stderr,
                )
                print(f"  line={line}", file=sys.stderr)
                return 5
        if any(pattern in line for pattern in error_patterns):
            print(f"[{case.case_id}] fail-fast error log detected.", file=sys.stderr)
            print(f"  line={line}", file=sys.stderr)
            return 6

        if "discord ingress parsed message" in line:
            state.seen_dispatch = True
        if "→ Bot:" in line:
            state.seen_bot = True
            state.bot_lines.append(line)
    return None
