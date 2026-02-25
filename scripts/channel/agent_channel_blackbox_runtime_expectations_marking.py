#!/usr/bin/env python3
"""Expectation match-state mutation helpers for blackbox probes."""

from __future__ import annotations

from typing import Any

from agent_channel_blackbox_runtime_expectations_targeting import (
    event_line_matches_target_recipient,
    observation_matches_target_recipient,
    reply_json_field_matches,
)


def mark_event_expectation(
    cfg: Any,
    state: Any,
    *,
    event_token: str,
    line: str,
    parse_log_tokens_fn: Any,
) -> None:
    """Update event expectation match state."""
    for index, expected in enumerate(cfg.expect_events):
        if (
            not state.matched_expect_events[index]
            and event_token == expected
            and event_line_matches_target_recipient(
                state, line, parse_log_tokens_fn=parse_log_tokens_fn
            )
        ):
            state.matched_expect_events[index] = True


def mark_reply_json_expectation(
    cfg: Any,
    state: Any,
    *,
    json_summary_obs: dict[str, str],
    target_session_scope_placeholder: str,
) -> None:
    """Update reply-json expectation match state."""
    json_summary_event = json_summary_obs.get("event")
    event_scoped_match = not cfg.expect_events or (json_summary_event in cfg.expect_events)
    if event_scoped_match and observation_matches_target_recipient(state, json_summary_obs):
        for index, (key, expected) in enumerate(cfg.expect_reply_json_fields):
            if state.matched_expect_reply_json_fields[index]:
                continue
            if reply_json_field_matches(
                key,
                expected,
                json_summary_obs,
                expected_session_scopes=state.expected_session_scopes,
                target_session_scope_placeholder=target_session_scope_placeholder,
            ):
                state.matched_expect_reply_json_fields[index] = True


def mark_expect_log_patterns(state: Any, line: str) -> None:
    """Update expect-log regex matches."""
    for index, pattern in enumerate(state.expect_log_compiled):
        if not state.matched_expect_log[index] and pattern.search(line):
            state.matched_expect_log[index] = True


def mark_expect_bot_patterns(state: Any, line: str) -> None:
    """Update expect-bot regex matches from one bot log line."""
    state.bot_observations.append(line)
    for index, pattern in enumerate(state.expect_bot_compiled):
        if not state.matched_expect_bot[index] and pattern.search(line):
            state.matched_expect_bot[index] = True
