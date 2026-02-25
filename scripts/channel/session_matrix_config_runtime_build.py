#!/usr/bin/env python3
"""Config construction helpers for session matrix runtime config."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def build_config(
    args: Any,
    *,
    config_cls: Any,
    resolve_runtime_partition_mode_fn: Any,
    group_profile_int_fn: Any,
    session_ids_from_runtime_log_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    expected_session_key_fn: Any,
) -> Any:
    """Validate and construct session matrix config."""
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be a positive integer.")
    if args.max_idle_secs <= 0:
        raise ValueError("--max-idle-secs must be a positive integer.")

    log_file = Path(args.log_file)
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

    thread_b = args.thread_b
    if thread_a is not None and thread_b is None:
        thread_b = int(thread_a) + 1
    thread_c = args.thread_c
    runtime_partition_mode = resolve_runtime_partition_mode_fn(log_file)
    if runtime_partition_mode == "chat_thread_user":
        if thread_a is None:
            thread_a = 0
        if thread_b is None:
            thread_b = 0
        if thread_c is None:
            thread_c = 0

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

    key_a = expected_session_key_fn(int(chat_id), int(user_a), thread_a, runtime_partition_mode)
    key_b = expected_session_key_fn(int(chat_b), int(user_b), thread_b, runtime_partition_mode)
    key_c = expected_session_key_fn(int(chat_c), int(user_c), thread_c, runtime_partition_mode)
    unique_keys = {key_a, key_b, key_c}
    if len(unique_keys) != 3:
        raise ValueError(
            "session matrix requires three distinct session identities "
            f"(got keys: {key_a}, {key_b}, {key_c}). Adjust chat/user/thread parameters."
        )

    username = args.username.strip() if args.username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)

    return config_cls(
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
        webhook_url=args.webhook_url,
        log_file=log_file,
        chat_id=int(chat_id),
        chat_b=int(chat_b),
        chat_c=int(chat_c),
        user_a=int(user_a),
        user_b=int(user_b),
        user_c=int(user_c),
        username=username,
        thread_a=thread_a,
        thread_b=thread_b,
        thread_c=thread_c,
        mixed_plain_prompt=args.mixed_plain_prompt.strip(),
        secret_token=(args.secret_token.strip() if args.secret_token else None),
        output_json=Path(args.output_json),
        output_markdown=Path(args.output_markdown),
        forbid_log_regexes=tuple(args.forbid_log_regex),
        session_partition=runtime_partition_mode,
    )
