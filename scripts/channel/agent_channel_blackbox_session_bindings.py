#!/usr/bin/env python3
"""Session-key helper bindings for channel blackbox probe."""

from __future__ import annotations

from typing import Any


def normalize_session_partition(value: str | None, *, normalize_partition_fn: Any) -> str | None:
    """Normalize session partition mode."""
    return normalize_partition_fn(value)


def expected_session_keys(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
    *,
    session_keys_module: Any,
    normalize_partition_fn: Any,
) -> tuple[str, ...]:
    """Build all acceptable session keys for one identity."""
    return session_keys_module.expected_session_keys(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_partition_fn,
    )


def expected_session_key(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
    *,
    session_keys_module: Any,
    normalize_partition_fn: Any,
) -> str:
    """Build canonical session key for one identity."""
    return session_keys_module.expected_session_key(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_partition_fn,
    )


def expected_session_scope_values(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None = None,
    scope_prefixes: tuple[str, ...] = ("telegram:",),
    *,
    session_keys_module: Any,
    normalize_partition_fn: Any,
) -> tuple[str, ...]:
    """Build expected session_scope candidates."""
    return session_keys_module.expected_session_scope_values(
        chat_id,
        user_id,
        thread_id,
        session_partition,
        normalize_partition_fn=normalize_partition_fn,
        scope_prefixes=scope_prefixes,
    )


def expected_session_scope_prefixes(
    expect_events: tuple[str, ...],
    *,
    session_keys_module: Any,
    telegram_prefix: str,
    discord_prefix: str,
) -> tuple[str, ...]:
    """Build allowed session-scope prefixes from expected events."""
    return session_keys_module.expected_session_scope_prefixes(
        expect_events,
        telegram_prefix=telegram_prefix,
        discord_prefix=discord_prefix,
    )


def expected_recipient_key(chat_id: int, thread_id: int | None) -> str:
    """Build expected recipient key."""
    if thread_id is None:
        return str(chat_id)
    return f"{chat_id}:{thread_id}"
