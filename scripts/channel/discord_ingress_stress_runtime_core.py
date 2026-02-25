#!/usr/bin/env python3
"""Core utility helpers for Discord ingress stress runtime."""

from __future__ import annotations

import os
import secrets
import time
from datetime import UTC, datetime
from typing import Any


def utc_now() -> str:
    """Return current UTC timestamp in RFC3339-like format."""
    return datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")


def p95(values: list[float]) -> float:
    """Compute simple p95 for small sample sets."""
    if not values:
        return 0.0
    ordered = sorted(values)
    index = max(0, int((len(ordered) * 0.95) - 1))
    return float(ordered[index])


def next_event_id() -> str:
    """Generate unique synthetic Discord ingress event id."""
    base_ms = int(time.time() * 1000)
    pid_component = os.getpid() % 10_000
    rand_component = secrets.randbelow(1000)
    return str((base_ms * 10_000_000) + (pid_component * 1000) + rand_component)


def init_log_offset(path: Any) -> int:
    """Initialize byte offset cursor for incremental log reads."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists():
        path.touch()
    return path.stat().st_size


def read_new_log_lines(path: Any, offset: int) -> tuple[int, list[str]]:
    """Read new log lines from byte offset cursor."""
    if not path.exists():
        return offset, []
    with path.open("rb") as handle:
        handle.seek(offset)
        chunk = handle.read()
        next_offset = handle.tell()
    if not chunk:
        return next_offset, []
    text = chunk.decode("utf-8", errors="replace")
    return next_offset, text.splitlines()
