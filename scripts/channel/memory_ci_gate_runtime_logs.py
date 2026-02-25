#!/usr/bin/env python3
"""Log polling and counting helpers for memory CI gate runtime."""

from __future__ import annotations

import re
import time
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import subprocess


def wait_for_log_regex(
    path: Any,
    pattern: str,
    *,
    timeout_secs: int,
    process: subprocess.Popen[str] | None = None,
    read_tail_fn: Any,
    init_log_cursor_fn: Any,
    read_new_log_lines_with_cursor_fn: Any,
) -> None:
    """Wait until log stream matches regex pattern or timeout/process exit occurs."""
    regex = re.compile(pattern)
    deadline = time.monotonic() + timeout_secs
    if path.exists() and regex.search(read_tail_fn(path)):
        return
    cursor = init_log_cursor_fn(path, kind="offset")
    while time.monotonic() < deadline:
        if process is not None and process.poll() is not None:
            tail = read_tail_fn(path)
            raise RuntimeError(
                f"runtime process exited before readiness check passed.\ntail:\n{tail}"
            )
        if path.exists():
            cursor, lines = read_new_log_lines_with_cursor_fn(path, cursor)
            if any(regex.search(line) for line in lines):
                return
        time.sleep(1.0)
    tail = read_tail_fn(path)
    raise RuntimeError(
        f"timed out waiting for log pattern: {pattern}\nlog_file={path}\ntail:\n{tail}"
    )


def read_tail(path: Any, *, max_lines: int, read_log_tail_text_fn: Any) -> str:
    """Read last lines from log file."""
    if not path.exists():
        return ""
    lines = read_log_tail_text_fn(path).splitlines()
    if len(lines) <= max_lines:
        return "\n".join(lines)
    return "\n".join(lines[-max_lines:])


def count_log_event(path: Any, event_name: str, *, iter_log_lines_fn: Any) -> int:
    """Count event occurrences in log stream."""
    if not path.exists():
        return 0
    pattern = re.compile(rf'event="?{re.escape(event_name)}"?')
    return sum(1 for line in iter_log_lines_fn(path) if pattern.search(line))
