#!/usr/bin/env python3
"""Identity-resolution helpers for blackbox config construction."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def resolve_session_identity(
    args: Any,
    *,
    log_file: Path,
    session_ids_from_runtime_log_fn: Any,
) -> tuple[int | None, int | None, int | None]:
    """Resolve chat/user/thread ids from args, env, and runtime logs."""
    chat_id = args.chat_id if args.chat_id is not None else None
    user_id = args.user_id if args.user_id is not None else None
    thread_id = args.thread_id if args.thread_id is not None else None

    if chat_id is None:
        env_chat = os.environ.get("OMNI_TEST_CHAT_ID")
        if env_chat:
            chat_id = int(env_chat)
    if user_id is None:
        env_user = os.environ.get("OMNI_TEST_USER_ID")
        if env_user:
            user_id = int(env_user)
    if thread_id is None:
        env_thread = os.environ.get("OMNI_TEST_THREAD_ID")
        if env_thread:
            thread_id = int(env_thread)

    if chat_id is None or user_id is None:
        inferred_chat, inferred_user, inferred_thread = session_ids_from_runtime_log_fn(log_file)
        if chat_id is None:
            chat_id = inferred_chat
        if user_id is None:
            user_id = inferred_user
        if thread_id is None:
            thread_id = inferred_thread
    return chat_id, user_id, thread_id


def resolve_username(
    args: Any, *, log_file: Path, username_from_settings_fn: Any, username_from_runtime_log_fn: Any
) -> str | None:
    """Resolve username from args, settings, then runtime logs."""
    username: str | None = args.username
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)
    return username
