#!/usr/bin/env python3
"""Core expectation checks for blackbox runtime probes."""

from __future__ import annotations

from typing import Any


def all_expectations_satisfied(state: Any) -> bool:
    """Check whether all configured expectations are satisfied."""
    return (
        all(state.matched_expect_events)
        and all(state.matched_expect_reply_json_fields)
        and all(state.matched_expect_log)
        and all(state.matched_expect_bot)
    )


def missing_expectations(cfg: Any, state: Any) -> tuple[list[str], list[str], list[str], list[str]]:
    """Return human-readable list of missing expectation tokens."""
    missing_events = [
        cfg.expect_events[index] for index, ok in enumerate(state.matched_expect_events) if not ok
    ]
    missing_reply_json_fields = [
        f"{key}={value}"
        for index, (key, value) in enumerate(cfg.expect_reply_json_fields)
        if not state.matched_expect_reply_json_fields[index]
    ]
    missing_log = [
        cfg.expect_log_regexes[index] for index, ok in enumerate(state.matched_expect_log) if not ok
    ]
    missing_bot = [
        cfg.expect_bot_regexes[index] for index, ok in enumerate(state.matched_expect_bot) if not ok
    ]
    return missing_events, missing_reply_json_fields, missing_log, missing_bot


def latest_json_summary_for_event(
    state: Any,
    event: str | None,
) -> dict[str, str] | None:
    """Return latest json summary for target event, if present."""
    if not state.json_reply_summary_observations:
        return None
    if event:
        for obs in reversed(state.json_reply_summary_observations):
            if obs.get("event") == event:
                return obs
    return state.json_reply_summary_observations[-1]


def event_matches_expectations(cfg: Any, event: str) -> bool:
    """Check whether event token is expected (or unconstrained)."""
    return not cfg.expect_events or event in cfg.expect_events
