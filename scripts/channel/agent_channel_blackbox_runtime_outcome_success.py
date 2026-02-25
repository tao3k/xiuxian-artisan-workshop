#!/usr/bin/env python3
"""Success-path outcome handling for agent channel blackbox probe."""

from __future__ import annotations

import sys
from typing import Any


def handle_success_outcome(
    *,
    cfg: Any,
    state: Any,
    finish_fn: Any,
    helpers_module: Any,
    bot_line: str,
) -> int:
    """Handle probe outcome when outbound bot log was observed."""
    missing_events, missing_reply_json_fields, missing_log, missing_bot = (
        helpers_module.missing_expectations(cfg, state)
    )
    if missing_events or missing_reply_json_fields or missing_log or missing_bot:
        if not missing_events and not missing_reply_json_fields and not missing_log and missing_bot:
            print(
                "Probe failed: outbound bot reply observed, but expect-bot regex did not match.",
                file=sys.stderr,
            )
            print(f"Missing expect-bot regex: {missing_bot}", file=sys.stderr)
            print("Observed outbound bot logs (latest first):", file=sys.stderr)
            for observed in reversed(state.bot_observations[-3:]):
                print(f"  {observed}", file=sys.stderr)
            return finish_fn(11)
        print("Probe failed: bot replied but expectations are incomplete.", file=sys.stderr)
        if missing_events:
            print(f"Missing expect-event values: {missing_events}", file=sys.stderr)
        if missing_reply_json_fields:
            print(
                f"Missing expect-reply-json-field values: {missing_reply_json_fields}",
                file=sys.stderr,
            )
        if missing_log:
            print(f"Missing expect-log regex: {missing_log}", file=sys.stderr)
        if missing_bot:
            print(f"Missing expect-bot regex: {missing_bot}", file=sys.stderr)
        return finish_fn(8)

    session_ok, mismatch_context = helpers_module.validate_target_session_scope(cfg, state)
    if not session_ok:
        print("Probe failed: command reply session_key mismatch.", file=sys.stderr)
        print(f"  expected_session_keys={list(state.expected_sessions)}", file=sys.stderr)
        print(f"  {mismatch_context}", file=sys.stderr)
        return finish_fn(10)

    target_obs = helpers_module.pick_target_command_reply_observation(cfg, state)
    print("Blackbox probe succeeded.")
    print("Observed outbound bot log:")
    print(f"  {bot_line}")
    if state.command_reply_observations:
        latest_obs = target_obs or state.command_reply_observations[-1]
        print("Reply observability:")
        print(f"  event={latest_obs.get('event')}")
        print(f"  session_key={latest_obs.get('session_key')}")
        print(f"  recipient={latest_obs.get('recipient')}")
        print(f"  reply_chars={latest_obs.get('reply_chars')}")
        print(f"  reply_bytes={latest_obs.get('reply_bytes')}")
        latest_summary = helpers_module.latest_json_summary_for_event(
            state,
            str(latest_obs.get("event")),
        )
    else:
        latest_summary = helpers_module.latest_json_summary_for_event(
            state,
            cfg.expect_events[0] if cfg.expect_events else None,
        )

    if latest_summary:
        print("Reply json summary:")
        for key in (
            "json_kind",
            "json_available",
            "json_status",
            "json_found",
            "json_decision",
            "json_session_scope",
            "json_keys",
        ):
            value = latest_summary.get(key)
            if value:
                print(f"  {key}={value}")
    return finish_fn(0)
