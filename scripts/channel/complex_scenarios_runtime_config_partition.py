#!/usr/bin/env python3
"""Runtime partition helpers for complex scenario runtime config."""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    env_get_fn: Any,
    normalize_partition_fn: Any,
    partition_mode_from_runtime_log_fn: Any,
    partition_mode_from_settings_fn: Any,
) -> str | None:
    """Resolve runtime partition mode from env override, log hints, then settings."""
    override = env_get_fn("OMNI_BLACKBOX_SESSION_PARTITION_MODE", "").strip()
    normalized_override = normalize_partition_fn(override)
    if normalized_override:
        return normalized_override

    mode_from_log = partition_mode_from_runtime_log_fn(log_file)
    if mode_from_log:
        return mode_from_log

    return partition_mode_from_settings_fn()


def apply_runtime_partition_defaults(
    sessions: tuple[Any, ...], partition_mode: str | None
) -> tuple[Any, ...]:
    """Apply partition-mode dependent identity defaults."""
    if partition_mode != "chat_thread_user":
        return sessions
    return tuple(
        session if session.thread_id is not None else replace(session, thread_id=0)
        for session in sessions
    )
