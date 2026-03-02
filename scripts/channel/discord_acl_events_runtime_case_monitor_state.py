#!/usr/bin/env python3
"""Mutable state model for Discord ACL runtime case monitoring."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class MonitorState:
    """In-flight observation state for one runtime case monitor loop."""

    last_log_activity: float
    seen_dispatch: bool
    seen_bot: bool
    bot_lines: list[str]
    command_reply_observations: list[dict[str, object]]
    json_reply_summary_observations: list[dict[str, str]]
    matched_expect_event: bool
    matched_expect_reply_json: list[bool]


def new_monitor_state(*, expect_fields_count: int, now: float) -> MonitorState:
    """Create initial monitor state for one case run."""
    return MonitorState(
        last_log_activity=now,
        seen_dispatch=False,
        seen_bot=False,
        bot_lines=[],
        command_reply_observations=[],
        json_reply_summary_observations=[],
        matched_expect_event=False,
        matched_expect_reply_json=[False] * expect_fields_count,
    )
