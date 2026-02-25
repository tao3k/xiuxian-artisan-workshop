#!/usr/bin/env python3
"""Env/runtime helpers for command-events runtime context."""

from __future__ import annotations

import os
from pathlib import Path
from typing import Any

RUNTIME_LOG_TAIL_BYTES = 256 * 1024


def parse_optional_int_env(var_name: str) -> int | None:
    """Parse optional integer environment variable."""
    raw = os.environ.get(var_name, "").strip()
    if not raw:
        return None
    try:
        return int(raw)
    except ValueError as error:
        raise ValueError(f"{var_name} must be an integer, got '{raw}'.") from error


def dedup_ints(values: list[int]) -> tuple[int, ...]:
    """Preserve-order integer de-dup."""
    ordered: list[int] = []
    for value in values:
        if value not in ordered:
            ordered.append(value)
    return tuple(ordered)


def runtime_log_file() -> Path:
    """Resolve runtime log file path from env."""
    return Path(os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log"))


def read_log_tail_lines(
    path: Path,
    *,
    read_log_tail_lines_fn: Any,
    tail_bytes: int = RUNTIME_LOG_TAIL_BYTES,
) -> list[str]:
    """Read runtime log tail lines with bounded bytes."""
    return read_log_tail_lines_fn(path, tail_bytes=tail_bytes)


def resolve_runtime_partition_mode(
    *,
    normalize_telegram_session_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve runtime partition mode from override/log/settings chain."""
    override = os.environ.get("OMNI_BLACKBOX_SESSION_PARTITION_MODE", "").strip()
    normalized_override = normalize_telegram_session_partition_mode_fn(override)
    if normalized_override:
        return normalized_override

    mode_from_log = session_partition_mode_from_runtime_log_fn(runtime_log_file())
    if mode_from_log:
        return mode_from_log

    return telegram_session_partition_mode_fn()
