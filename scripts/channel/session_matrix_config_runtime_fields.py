#!/usr/bin/env python3
"""Structured result-field helpers for session matrix runtime config."""

from __future__ import annotations

from typing import Any


def session_context_result_fields(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    expected_session_key_fn: Any,
) -> tuple[str, ...]:
    """Build expected JSON fields for `/session json` result."""
    session_key = expected_session_key_fn(chat_id, user_id, thread_id, session_partition)
    return (
        "json_kind=session_context",
        f"json_logical_session_id=telegram:{session_key}",
        f"json_partition_key={session_key}",
    )


def session_memory_result_fields(
    chat_id: int,
    user_id: int,
    thread_id: int | None,
    session_partition: str | None,
    *,
    expected_session_key_fn: Any,
) -> tuple[str, ...]:
    """Build expected JSON fields for `/session memory json` result."""
    session_key = expected_session_key_fn(chat_id, user_id, thread_id, session_partition)
    return (
        "json_kind=session_memory",
        f"json_session_scope=telegram:{session_key}",
    )
