#!/usr/bin/env python3
"""Session-key bindings for complex-scenarios runtime entrypoint."""

from __future__ import annotations

from typing import Any


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    partition_mode: str | None = None,
    *,
    session_keys_module: Any,
    normalize_partition_fn: Any,
) -> tuple[str, ...]:
    """Build all acceptable session keys for one session identity."""
    return session_keys_module.expected_session_keys(
        chat_id,
        user_id,
        thread_id,
        partition_mode,
        normalize_partition_fn=normalize_partition_fn,
    )


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    partition_mode: str | None = None,
    *,
    session_keys_module: Any,
    normalize_partition_fn: Any,
) -> str:
    """Build canonical session key for one session identity."""
    return session_keys_module.expected_session_key(
        chat_id,
        user_id,
        thread_id,
        partition_mode,
        normalize_partition_fn=normalize_partition_fn,
    )


def expected_session_log_regex(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    partition_mode: str | None = None,
    *,
    session_keys_module: Any,
    normalize_partition_fn: Any,
) -> str:
    """Build session-key regex expected in runtime logs."""
    return session_keys_module.expected_session_log_regex(
        chat_id,
        user_id,
        thread_id,
        partition_mode,
        normalize_partition_fn=normalize_partition_fn,
    )
