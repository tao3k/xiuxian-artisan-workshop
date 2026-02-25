#!/usr/bin/env python3
"""Report payload helpers for runtime monitor."""

from __future__ import annotations

import signal
from typing import Any


def build_spawn_error_report(
    *,
    start_wall: str,
    end_wall: str,
    command: list[str],
    error: OSError,
) -> dict[str, Any]:
    """Build report payload when process spawn fails."""
    return {
        "schema_version": 1,
        "start_time_utc": start_wall,
        "end_time_utc": end_wall,
        "duration_ms": 0,
        "command": command,
        "pid": None,
        "exit": {"kind": "spawn_error", "exit_code": 127, "signal": None, "signal_name": None},
        "spawn_error": str(error),
        "stats": {},
        "events": {"last_event": None, "counts": {}},
        "tail": [],
    }


def termination_payload(requested_signal: int) -> dict[str, Any]:
    """Build structured termination signal payload."""
    try:
        signal_name = signal.Signals(requested_signal).name
    except ValueError:
        signal_name = f"SIG{requested_signal}"
    return {
        "requested_signal": requested_signal,
        "requested_signal_name": signal_name,
    }


def build_runtime_report(
    *,
    start_wall: str,
    end_wall: str,
    duration_ms: int,
    command: list[str],
    pid: int | None,
    returncode: int,
    exit_info: dict[str, Any],
    stats: Any,
    event_counts: Any,
    recent_lines: Any,
    cwd: str,
    log_file: str,
    requested_signal: int | None,
) -> dict[str, Any]:
    """Build final monitor report payload."""
    report: dict[str, Any] = {
        "schema_version": 1,
        "start_time_utc": start_wall,
        "end_time_utc": end_wall,
        "duration_ms": duration_ms,
        "command": command,
        "pid": pid,
        "returncode_raw": returncode,
        "exit": exit_info,
        "stats": {
            "total_lines": stats.total_lines,
            "error_lines": stats.error_lines,
            "first_error_line": stats.first_error_line,
            "saw_webhook": stats.saw_webhook,
            "saw_user_dispatch": stats.saw_user_dispatch,
            "saw_bot_reply": stats.saw_bot_reply,
        },
        "events": {
            "last_event": stats.last_event,
            "counts": dict(event_counts.most_common(20)),
        },
        "tail": list(recent_lines),
        "environment": {
            "cwd": cwd,
            "log_file": log_file,
        },
    }
    if requested_signal is not None:
        report["termination"] = termination_payload(requested_signal)
    return report
