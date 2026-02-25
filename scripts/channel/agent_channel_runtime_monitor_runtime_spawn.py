#!/usr/bin/env python3
"""Spawn + spawn-error helpers for runtime monitor."""

from __future__ import annotations

import subprocess
import sys
from typing import Any

from agent_channel_runtime_monitor_common import now_utc_iso, write_report
from agent_channel_runtime_monitor_runtime_report import build_spawn_error_report


def spawn_monitored_process(command: list[str]) -> subprocess.Popen[str]:
    """Spawn monitored process with merged stdout/stderr streaming."""
    return subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )


def write_spawn_failure_report(
    *,
    command: list[str],
    report_file: Any,
    report_jsonl: Any,
    error: OSError,
) -> int:
    """Persist spawn failure report and return canonical exit code."""
    start_wall = now_utc_iso()
    end_wall = now_utc_iso()
    report = build_spawn_error_report(
        start_wall=start_wall,
        end_wall=end_wall,
        command=command,
        error=error,
    )
    write_report(report_file, report_jsonl, report)
    print(
        f"[monitor] spawn failed: {error}. report={report_file}",
        file=sys.stderr,
        flush=True,
    )
    return 127
