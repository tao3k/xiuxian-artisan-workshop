#!/usr/bin/env python3
"""Runtime support bindings for memory benchmark entrypoint."""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Any


def resolve_runtime_partition_mode(
    log_file: Any,
    *,
    config_module: Any,
    normalize_telegram_session_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve runtime partition mode from settings/runtime log."""
    return config_module.resolve_runtime_partition_mode(
        log_file,
        normalize_telegram_session_partition_mode_fn=normalize_telegram_session_partition_mode_fn,
        session_partition_mode_from_runtime_log_fn=session_partition_mode_from_runtime_log_fn,
        telegram_session_partition_mode_fn=telegram_session_partition_mode_fn,
    )


def count_lines(path: Any, *, init_log_cursor_fn: Any) -> int:
    """Count lines by offset cursor snapshot."""
    return init_log_cursor_fn(path, kind="offset").value


def read_new_lines(
    path: Any,
    cursor: int,
    *,
    read_new_log_lines_with_cursor_fn: Any,
    log_cursor_cls: Any,
) -> tuple[int, list[str]]:
    """Read newly appended lines from offset cursor."""
    next_cursor, lines = read_new_log_lines_with_cursor_fn(
        path,
        log_cursor_cls(kind="offset", value=cursor),
    )
    return next_cursor.value, lines


def to_iso_utc(unix_ts: float) -> str:
    """Convert unix timestamp to ISO8601 UTC string."""
    return datetime.fromtimestamp(unix_ts, tz=UTC).isoformat()
