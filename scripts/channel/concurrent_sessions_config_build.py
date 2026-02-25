#!/usr/bin/env python3
"""Config builder for concurrent Telegram session probes."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    import argparse

    from concurrent_sessions_models import ProbeConfig


def build_config(
    args: argparse.Namespace,
    *,
    group_profile_int_fn: Any,
    session_ids_from_runtime_log_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    telegram_webhook_secret_token_fn: Any,
    expected_session_keys_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
    probe_config_cls: type[ProbeConfig],
) -> ProbeConfig:
    """Build validated probe config from parsed args + resolver dependencies."""
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be a positive integer.")

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

    user_b = args.user_b
    if user_b is None:
        user_b = group_profile_int_fn("OMNI_TEST_USER_B")
    if user_b is None:
        user_b = int(user_a) + 1

    username = args.username.strip() if args.username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)

    runtime_partition_mode = resolve_runtime_partition_mode_fn(
        log_file,
        override=getattr(args, "session_partition", None),
    )

    key_a_candidates = set(
        expected_session_keys_fn(int(chat_id), int(user_a), thread_a, runtime_partition_mode)
    )
    key_b_candidates = set(
        expected_session_keys_fn(int(chat_b), int(user_b), args.thread_b, runtime_partition_mode)
    )
    if key_a_candidates & key_b_candidates:
        raise ValueError(
            "session-a and session-b resolve to the same session_key; adjust "
            "--chat-b/--user-b/--thread-b to target distinct sessions "
            f"(partition={runtime_partition_mode or 'unknown'})."
        )

    secret_token: str | None = args.secret_token.strip() if args.secret_token else None
    if not secret_token:
        secret_token = telegram_webhook_secret_token_fn()

    return probe_config_cls(
        max_wait=args.max_wait,
        webhook_url=args.webhook_url,
        log_file=log_file,
        chat_id=int(chat_id),
        chat_b=int(chat_b),
        user_a=int(user_a),
        user_b=int(user_b),
        username=username,
        thread_a=thread_a,
        thread_b=args.thread_b,
        secret_token=secret_token,
        prompt=args.prompt,
        forbid_log_regexes=tuple(
            pattern
            for pattern in args.forbid_log_regex
            if not (args.allow_send_failure and pattern == "Telegram sendMessage failed")
        ),
        allow_send_failure=bool(args.allow_send_failure),
        session_partition=runtime_partition_mode,
    )
