#!/usr/bin/env python3
"""Failure-path outcome handling for agent channel blackbox probe."""

from __future__ import annotations

import sys
from typing import Any


def handle_no_bot_outcome(
    *,
    cfg: Any,
    state: Any,
    finish_fn: Any,
    tail_lines_fn: Any,
    helpers_module: Any,
    trace_mode: bool,
    seen_trace: bool,
    seen_user_dispatch: bool,
    error_line: str,
    webhook_seen: bool,
    trace_id: str,
) -> int:
    """Handle timeout/failure outcomes when no outbound bot log was seen."""
    if trace_mode and not seen_trace:
        if webhook_seen:
            print(
                "Probe timed out: webhook update arrived, but trace marker was not observed in downstream logs.",
                file=sys.stderr,
            )
        else:
            print("Probe timed out: did not observe inbound marker in logs.", file=sys.stderr)
        print(f"Expected marker: {trace_id}", file=sys.stderr)
        print(
            "Tip: ensure runtime log file is correct and webhook process is running.",
            file=sys.stderr,
        )
        last = tail_lines_fn(cfg.log_file, 40)
        if last:
            print("Last 40 log lines:", file=sys.stderr)
            for line in last:
                print(f"  {line}", file=sys.stderr)
        return finish_fn(2)

    if not seen_user_dispatch:
        print(
            "Probe timed out: trace marker observed, but dispatch marker `← User:` was not observed.",
            file=sys.stderr,
        )
        print(f"Expected dispatch trace: [{trace_id}]", file=sys.stderr)
        last = tail_lines_fn(cfg.log_file, 60)
        if last:
            print("Last 60 log lines:", file=sys.stderr)
            for line in last:
                print(f"  {line}", file=sys.stderr)
        return finish_fn(9)

    print(
        "Probe timed out: inbound marker observed, but no outbound bot log found before max-wait.",
        file=sys.stderr,
    )
    missing_events, missing_reply_json_fields, missing_log, missing_bot = (
        helpers_module.missing_expectations(cfg, state)
    )
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
    if error_line:
        print("Last related error:", file=sys.stderr)
        print(f"  {error_line}", file=sys.stderr)
    if state.command_reply_observations:
        print("Observed command reply events (latest first):", file=sys.stderr)
        for obs in reversed(state.command_reply_observations[-3:]):
            print(
                "  "
                f"event={obs.get('event')} "
                f"session_key={obs.get('session_key')} "
                f"recipient={obs.get('recipient')} "
                f"reply_chars={obs.get('reply_chars')} "
                f"reply_bytes={obs.get('reply_bytes')}",
                file=sys.stderr,
            )
    if state.json_reply_summary_observations:
        print("Observed reply json summary events (latest first):", file=sys.stderr)
        for obs in reversed(state.json_reply_summary_observations[-3:]):
            print(
                "  "
                f"event={obs.get('event')} "
                f"json_kind={obs.get('json_kind')} "
                f"json_available={obs.get('json_available')} "
                f"json_status={obs.get('json_status')} "
                f"json_found={obs.get('json_found')} "
                f"json_decision={obs.get('json_decision')} "
                f"json_keys={obs.get('json_keys')}",
                file=sys.stderr,
            )
    last = tail_lines_fn(cfg.log_file, 60)
    if last:
        print("Last 60 log lines:", file=sys.stderr)
        for line in last:
            print(f"  {line}", file=sys.stderr)
    return finish_fn(3)
