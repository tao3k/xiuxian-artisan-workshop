#!/usr/bin/env python3
"""Output streaming helpers for runtime monitor."""

from __future__ import annotations

import contextlib
import signal
import subprocess
from typing import Any

from agent_channel_runtime_monitor_runtime_stream import process_stream_line


def stream_process_output(
    proc: subprocess.Popen[str],
    *,
    output: Any,
    stats: Any,
    recent_lines: Any,
    event_counts: Any,
    error_markers: tuple[str, ...],
    extract_event_token_fn: Any,
) -> bool:
    """Stream process output into stdout/log while collecting monitor stats."""
    interrupted = False
    try:
        assert proc.stdout is not None
        for line in proc.stdout:
            line = line.rstrip("\n")
            print(line, flush=True)
            output.write(line + "\n")
            output.flush()

            process_stream_line(
                line,
                stats=stats,
                recent_lines=recent_lines,
                event_counts=event_counts,
                error_markers=error_markers,
                extract_event_token_fn=extract_event_token_fn,
            )
    except KeyboardInterrupt:
        interrupted = True
        with contextlib.suppress(ProcessLookupError):
            proc.send_signal(signal.SIGINT)
    return interrupted


def wait_for_process(proc: subprocess.Popen[str], *, interrupted: bool) -> int:
    """Wait for process completion, handling interrupted timeout fallback."""
    if interrupted:
        try:
            return proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()
            return proc.wait()
    return proc.wait()
