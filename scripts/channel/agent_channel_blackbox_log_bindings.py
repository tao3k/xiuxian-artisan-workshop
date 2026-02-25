#!/usr/bin/env python3
"""Log cursor bindings for channel blackbox probe."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def count_lines(path: Path, *, init_log_cursor_fn: Any) -> int:
    """Count current log lines via shared cursor offset."""
    return init_log_cursor_fn(path, kind="offset").value


def read_new_lines(
    path: Path,
    cursor: int,
    *,
    read_new_log_lines_with_cursor_fn: Any,
    shared_log_cursor_cls: Any,
) -> tuple[int, list[str]]:
    """Read new log lines since cursor and return next cursor with chunk."""
    next_cursor, lines = read_new_log_lines_with_cursor_fn(
        path,
        shared_log_cursor_cls(kind="offset", value=cursor),
    )
    return next_cursor.value, lines


def tail_lines(path: Path, n: int, *, tail_log_lines_fn: Any) -> list[str]:
    """Read last N lines from log via shared tail helper."""
    return tail_log_lines_fn(path, n)
