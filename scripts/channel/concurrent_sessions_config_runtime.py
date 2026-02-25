#!/usr/bin/env python3
"""Runtime partition and session-key helpers for concurrent session probes."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_runtime_partition_mode(
    log_file: Path,
    *,
    override: str | None,
    normalize_partition_mode_fn: Any,
    session_partition_mode_from_runtime_log_fn: Any,
    telegram_session_partition_mode_fn: Any,
) -> str | None:
    """Resolve effective runtime partition mode from override/log/settings."""
    normalized_override = normalize_partition_mode_fn(override)
    if normalized_override:
        return normalized_override

    mode_from_log = session_partition_mode_from_runtime_log_fn(log_file)
    if mode_from_log:
        return mode_from_log
    return telegram_session_partition_mode_fn()


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    expected_session_keys_fn: Any,
    normalize_partition_mode_fn: Any,
) -> tuple[str, ...]:
    """Return accepted session-key aliases for one identity."""
    return expected_session_keys_fn(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_partition_mode_fn,
    )


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    expected_session_key_fn: Any,
    normalize_partition_mode_fn: Any,
) -> str:
    """Return canonical session key for one identity."""
    return expected_session_key_fn(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_partition_mode_fn,
    )
