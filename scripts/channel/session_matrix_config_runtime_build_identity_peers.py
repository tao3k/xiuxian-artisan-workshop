#!/usr/bin/env python3
"""Peer identity resolution helpers for session matrix runtime config build."""

from __future__ import annotations

from typing import Any


def resolve_peer_chats(args: Any, *, chat_id: int, group_profile_int_fn: Any) -> tuple[int, int]:
    """Resolve chat ids for session B and C."""
    chat_b = args.chat_b
    if chat_b is None:
        chat_b = group_profile_int_fn("OMNI_TEST_CHAT_B")
    if chat_b is None:
        chat_b = int(chat_id)

    chat_c = args.chat_c
    if chat_c is None:
        chat_c = group_profile_int_fn("OMNI_TEST_CHAT_C")
    if chat_c is None:
        chat_c = int(chat_id)

    return int(chat_b), int(chat_c)


def resolve_threads(
    args: Any,
    *,
    thread_a: int | None,
    runtime_partition_mode: str | None,
) -> tuple[int | None, int | None, int | None]:
    """Resolve thread ids based on args and runtime partition mode."""
    thread_b = args.thread_b
    if thread_a is not None and thread_b is None:
        thread_b = int(thread_a) + 1

    thread_c = args.thread_c
    if runtime_partition_mode == "chat_thread_user":
        if thread_a is None:
            thread_a = 0
        if thread_b is None:
            thread_b = 0
        if thread_c is None:
            thread_c = 0
    return thread_a, thread_b, thread_c


def resolve_peer_users(
    args: Any,
    *,
    chat_id: int,
    chat_b: int,
    user_a: int,
    thread_a: int | None,
    thread_b: int | None,
    group_profile_int_fn: Any,
) -> tuple[int, int]:
    """Resolve session B/C users with cross-thread fallback rules."""
    user_b = args.user_b
    if user_b is None:
        user_b = group_profile_int_fn("OMNI_TEST_USER_B")
    if user_b is None:
        if (
            int(chat_b) == int(chat_id)
            and thread_a is not None
            and thread_b is not None
            and int(thread_a) != int(thread_b)
        ):
            user_b = int(user_a)
        else:
            user_b = int(user_a) + 1

    user_c = args.user_c
    if user_c is None:
        user_c = group_profile_int_fn("OMNI_TEST_USER_C")
    if user_c is None:
        user_c = int(user_a) + 2

    return int(user_b), int(user_c)
