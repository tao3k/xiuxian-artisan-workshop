#!/usr/bin/env python3
"""Session assembly and uniqueness validation for complex runtime config."""

from __future__ import annotations

from typing import Any


def build_sessions(
    args: Any,
    *,
    session_identity_cls: Any,
    chat_a: int,
    user_a: int,
    thread_a: int | None,
    chat_b: int,
    user_b: int,
    thread_b: int | None,
    chat_c: int,
    user_c: int,
    thread_c: int | None,
) -> tuple[Any, Any, Any]:
    """Build raw a/b/c session identities from resolved values."""
    return (
        session_identity_cls(
            alias="a",
            chat_id=chat_a,
            user_id=user_a,
            thread_id=thread_a,
            chat_title=(args.chat_title_a.strip() if args.chat_title_a else None),
        ),
        session_identity_cls(
            alias="b",
            chat_id=chat_b,
            user_id=user_b,
            thread_id=thread_b,
            chat_title=(args.chat_title_b.strip() if args.chat_title_b else None),
        ),
        session_identity_cls(
            alias="c",
            chat_id=chat_c,
            user_id=user_c,
            thread_id=thread_c,
            chat_title=(args.chat_title_c.strip() if args.chat_title_c else None),
        ),
    )


def ensure_distinct_session_identity(
    sessions: tuple[Any, Any, Any],
    *,
    runtime_partition_mode: str | None,
    expected_session_keys_fn: Any,
    expected_session_key_fn: Any,
) -> None:
    """Ensure a/b/c sessions map to distinct storage identities."""
    key_sets = [
        set(
            expected_session_keys_fn(
                session.chat_id,
                session.user_id,
                session.thread_id,
                runtime_partition_mode,
            )
        )
        for session in sessions
    ]
    if key_sets[0] & key_sets[1] or key_sets[0] & key_sets[2] or key_sets[1] & key_sets[2]:
        keys = [
            expected_session_key_fn(
                session.chat_id,
                session.user_id,
                session.thread_id,
                runtime_partition_mode,
            )
            for session in sessions
        ]
        raise ValueError(
            "sessions a/b/c must map to distinct identities. "
            f"got keys={keys}; adjust chat/user/thread values."
        )
