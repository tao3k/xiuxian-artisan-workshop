#!/usr/bin/env python3
"""Validation and identity derivation helpers for complex runtime config."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def validate_runtime_args(args: Any) -> None:
    """Validate numeric runtime CLI bounds."""
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be positive")
    if args.max_idle_secs <= 0:
        raise ValueError("--max-idle-secs must be positive")
    if args.max_parallel <= 0:
        raise ValueError("--max-parallel must be positive")


def resolve_required_paths(args: Any) -> tuple[Path, Path, Path]:
    """Resolve required log/dataset/script paths and ensure they exist."""
    log_file = Path(args.log_file)
    dataset_path = Path(args.dataset)
    blackbox_script = Path(args.blackbox_script)
    if not dataset_path.exists():
        raise ValueError(f"dataset not found: {dataset_path}")
    if not blackbox_script.exists():
        raise ValueError(f"blackbox script not found: {blackbox_script}")
    return log_file, dataset_path, blackbox_script


def resolve_primary_identity(
    args: Any,
    *,
    log_file: Path,
    session_ids_from_runtime_log_fn: Any,
) -> tuple[int, int, int | None]:
    """Resolve primary chat/user/thread from args with runtime-log fallback."""
    chat_a = args.chat_a
    user_a = args.user_a
    thread_a = args.thread_a
    if chat_a is None or user_a is None:
        inferred_chat, inferred_user, inferred_thread = session_ids_from_runtime_log_fn(log_file)
        if chat_a is None:
            chat_a = inferred_chat
        if user_a is None:
            user_a = inferred_user
        if thread_a is None:
            thread_a = inferred_thread

    if chat_a is None or user_a is None:
        raise ValueError(
            "chat_a/user_a are required. Use --chat-a/--user-a or emit one live message first "
            "so ids can be inferred from runtime logs."
        )
    return int(chat_a), int(user_a), thread_a


def resolve_peer_user_ids(
    args: Any,
    *,
    user_a_int: int,
    parse_numeric_user_ids_fn: Any,
    pick_default_peer_user_id_fn: Any,
    allowed_users_from_settings_fn: Any,
) -> tuple[int, int]:
    """Resolve peer user IDs with settings-aware defaults."""
    allowlisted_numeric_users = parse_numeric_user_ids_fn(allowed_users_from_settings_fn())
    used_users = {user_a_int}

    if args.user_b is not None:
        user_b = int(args.user_b)
    else:
        user_b = pick_default_peer_user_id_fn(
            primary_user=user_a_int,
            preferred_offset=1,
            used=used_users,
            allowlisted_numeric_ids=allowlisted_numeric_users,
        )
    used_users.add(user_b)

    if args.user_c is not None:
        user_c = int(args.user_c)
    else:
        user_c = pick_default_peer_user_id_fn(
            primary_user=user_a_int,
            preferred_offset=2,
            used=used_users,
            allowlisted_numeric_ids=allowlisted_numeric_users,
        )
    return user_b, user_c
