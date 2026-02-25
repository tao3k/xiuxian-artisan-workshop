#!/usr/bin/env python3
"""Log cursor and ANSI helpers for concurrent session probes."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import re


def strip_ansi(value: str, *, ansi_escape_re: re.Pattern[str]) -> str:
    """Strip ANSI escape codes from one log line."""
    return ansi_escape_re.sub("", value)


def count_lines(path: Any, *, init_log_cursor_fn: Any) -> int:
    """Initialize offset cursor for incremental log polling."""
    return init_log_cursor_fn(path, kind="offset").value


def read_new_lines(
    path: Any,
    cursor: int,
    *,
    log_cursor_cls: Any,
    read_new_log_lines_with_cursor_fn: Any,
) -> tuple[int, list[str]]:
    """Read incremental log lines and return next cursor value."""
    next_cursor, lines = read_new_log_lines_with_cursor_fn(
        path,
        log_cursor_cls(kind="offset", value=cursor),
    )
    return next_cursor.value, lines
