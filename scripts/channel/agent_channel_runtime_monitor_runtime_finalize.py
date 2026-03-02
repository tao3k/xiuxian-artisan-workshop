#!/usr/bin/env python3
"""Finalize runtime monitor execution and persist report artifacts."""

from __future__ import annotations

import os
import sys
import time
from typing import Any


def finalize_runtime_report(
    *,
    start_wall: str,
    start_monotonic: float,
    command: list[str],
    pid: int | None,
    returncode: int,
    stats: Any,
    event_counts: Any,
    recent_lines: Any,
    log_file: Any,
    termination: Any,
    report_file: Any,
    report_jsonl: Any,
    classify_exit_fn: Any,
    now_utc_iso_fn: Any,
    build_runtime_report_fn: Any,
    write_report_fn: Any,
    normalize_exit_code_fn: Any,
) -> int:
    """Build/write runtime monitor report and return normalized exit code."""
    duration_ms = int((time.monotonic() - start_monotonic) * 1000)
    end_wall = now_utc_iso_fn()
    exit_info = classify_exit_fn(returncode)
    report = build_runtime_report_fn(
        start_wall=start_wall,
        end_wall=end_wall,
        duration_ms=duration_ms,
        command=command,
        pid=pid,
        returncode=returncode,
        exit_info=exit_info,
        stats=stats,
        event_counts=event_counts,
        recent_lines=recent_lines,
        cwd=os.getcwd(),
        log_file=str(log_file),
        requested_signal=termination.requested_signal,
    )
    write_report_fn(report_file, report_jsonl, report)

    print(
        "[monitor] "
        f"exit_kind={exit_info['kind']} "
        f"exit_code={exit_info['exit_code']} "
        f"signal={exit_info['signal_name'] or ''} "
        f"duration_ms={duration_ms} "
        f"report={report_file}",
        file=sys.stderr,
        flush=True,
    )
    if stats.first_error_line:
        print(
            f"[monitor] first_error_line={stats.first_error_line}",
            file=sys.stderr,
            flush=True,
        )

    return normalize_exit_code_fn(returncode)
