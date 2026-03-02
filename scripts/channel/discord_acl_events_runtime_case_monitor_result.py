#!/usr/bin/env python3
"""Result-construction helpers for Discord ACL runtime case monitoring."""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from discord_acl_events_runtime_case_monitor_state import MonitorState


def build_success_payload(state: MonitorState) -> dict[str, Any]:
    """Build stable payload consumed by downstream validation stages."""
    return {
        "seen_bot": state.seen_bot,
        "bot_lines": state.bot_lines,
        "command_reply_observations": state.command_reply_observations,
        "json_reply_summary_observations": state.json_reply_summary_observations,
    }


def finalize_monitor(
    *,
    state: MonitorState,
    case: Any,
    expect_fields: tuple[tuple[str, str], ...],
) -> tuple[int, dict[str, Any]]:
    """Finalize monitor result after loop completes or timeout occurs."""
    if not state.seen_dispatch:
        print(f"[{case.case_id}] timed out: no discord ingress dispatch marker.", file=sys.stderr)
        return 9, {}
    if not state.matched_expect_event or not all(state.matched_expect_reply_json):
        print(f"[{case.case_id}] timed out: missing expected event/json fields.", file=sys.stderr)
        print(f"  expect_event={case.event_name}", file=sys.stderr)
        if expect_fields:
            print(
                "  expect_reply_json=" + ",".join(f"{key}={value}" for key, value in expect_fields),
                file=sys.stderr,
            )
        return 8, {}

    return 0, build_success_payload(state)
