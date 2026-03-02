#!/usr/bin/env python3
"""Primary identity resolution for session matrix runtime config build."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_primary_identity(
    args: Any,
    *,
    log_file: Path,
    group_profile_int_fn: Any,
    session_ids_from_runtime_log_fn: Any,
) -> tuple[int, int, int | None]:
    """Resolve primary chat/user/thread identity from args/env/log."""
    chat_id = args.chat_id
    if chat_id is None:
        chat_id = group_profile_int_fn("OMNI_TEST_CHAT_ID")

    user_a = args.user_a
    if user_a is None:
        user_a = group_profile_int_fn("OMNI_TEST_USER_ID")

    thread_a = args.thread_a

    if chat_id is None or user_a is None:
        inferred_chat, inferred_user, inferred_thread = session_ids_from_runtime_log_fn(log_file)
        if chat_id is None:
            chat_id = inferred_chat
        if user_a is None:
            user_a = inferred_user
        if thread_a is None:
            thread_a = inferred_thread

    if chat_id is None or user_a is None:
        raise ValueError(
            "chat/user are required. Use --chat-id/--user-a "
            "(or OMNI_TEST_CHAT_ID/OMNI_TEST_USER_ID). "
            "Tip: send one Telegram message first so session_key can be inferred from logs."
        )
    return int(chat_id), int(user_a), thread_a
