#!/usr/bin/env python3
"""Compatibility facade for runtime monitor process execution/reporting."""

from __future__ import annotations

import os
import sys
import time
from collections import Counter, deque
from typing import TYPE_CHECKING, Any

from agent_channel_runtime_monitor_common import (
    classify_exit,
    extract_event_token,
    normalize_exit_code,
    now_utc_iso,
    write_report,
)
from agent_channel_runtime_monitor_models import (
    ERROR_MARKERS,
    MonitorStats,
    MonitorTerminationState,
)
from agent_channel_runtime_monitor_runtime_report import (
    build_runtime_report,
    termination_payload,
)
from agent_channel_runtime_monitor_runtime_signals import (
    install_termination_handlers,
    restore_signal_handlers,
)
from agent_channel_runtime_monitor_runtime_spawn import (
    spawn_monitored_process,
    write_spawn_failure_report,
)
from agent_channel_runtime_monitor_runtime_streaming import (
    stream_process_output,
    wait_for_process,
)

if TYPE_CHECKING:
    from pathlib import Path


def _build_spawn_error_report(
    *,
    start_wall: str,
    end_wall: str,
    command: list[str],
    error: OSError,
) -> dict[str, Any]:
    from agent_channel_runtime_monitor_runtime_report import build_spawn_error_report

    return build_spawn_error_report(
        start_wall=start_wall,
        end_wall=end_wall,
        command=command,
        error=error,
    )


def _termination_payload(requested_signal: int) -> dict[str, Any]:
    return termination_payload(requested_signal)


def run_monitored_process(
    command: list[str],
    log_file: Path,
    report_file: Path,
    report_jsonl: Path | None,
    tail_lines: int,
) -> int:
    """Run monitored process, stream logs, and persist structured report."""
    start_wall = now_utc_iso()
    start_monotonic = time.monotonic()
    stats = MonitorStats()
    termination = MonitorTerminationState()
    event_counts: Counter[str] = Counter()
    recent_lines: deque[str] = deque(maxlen=max(tail_lines, 1))
    pid: int | None = None

    log_file.parent.mkdir(parents=True, exist_ok=True)
    try:
        proc = spawn_monitored_process(command)
        pid = proc.pid
    except OSError as error:
        return write_spawn_failure_report(
            command=command,
            report_file=report_file,
            report_jsonl=report_jsonl,
            error=error,
        )

    previous_signal_handlers: dict[int, Any] = install_termination_handlers(proc, termination)

    with log_file.open("a", encoding="utf-8") as output:
        interrupted = stream_process_output(
            proc,
            output=output,
            stats=stats,
            recent_lines=recent_lines,
            event_counts=event_counts,
            error_markers=ERROR_MARKERS,
            extract_event_token_fn=extract_event_token,
        )
    restore_signal_handlers(previous_signal_handlers)

    returncode = wait_for_process(proc, interrupted=interrupted)

    duration_ms = int((time.monotonic() - start_monotonic) * 1000)
    end_wall = now_utc_iso()
    exit_info = classify_exit(returncode)
    report = build_runtime_report(
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
    write_report(report_file, report_jsonl, report)

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
        print(f"[monitor] first_error_line={stats.first_error_line}", file=sys.stderr, flush=True)

    return normalize_exit_code(returncode)
