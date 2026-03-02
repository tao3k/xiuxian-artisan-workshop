#!/usr/bin/env python3
"""Config construction helpers for agent channel blackbox."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from agent_channel_blackbox_config_build_identity import resolve_session_identity, resolve_username
from agent_channel_blackbox_config_build_values import resolve_allow_chat_ids, resolve_wait_secs


def build_config(
    args: Any,
    *,
    probe_config_cls: Any,
    session_ids_from_runtime_log_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    parse_expected_field_fn: Any,
    parse_allow_chat_ids_fn: Any,
    normalize_session_partition_fn: Any,
    telegram_webhook_secret_token_fn: Any,
) -> Any:
    """Build ProbeConfig from parsed args + environment fallbacks."""
    log_file = Path(args.log_file)
    chat_id, user_id, thread_id = resolve_session_identity(
        args,
        log_file=log_file,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
    )
    username = resolve_username(
        args,
        log_file=log_file,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
    )

    if chat_id is None or user_id is None:
        raise ValueError(
            "chat/user id are required. Set --chat-id/--user-id (or OMNI_TEST_CHAT_ID/OMNI_TEST_USER_ID). "
            f"Tip: run one real Telegram message first to auto-infer from {log_file}."
        )

    max_wait_secs = resolve_wait_secs(
        args.max_wait if args.max_wait is not None else args.timeout,
        fallback_env="OMNI_BLACKBOX_MAX_WAIT_SECS",
    )
    max_idle_secs = resolve_wait_secs(
        args.max_idle_secs,
        fallback_env="OMNI_BLACKBOX_MAX_IDLE_SECS",
    )
    expect_reply_json_fields = tuple(
        parse_expected_field_fn(value) for value in args.expect_reply_json_field
    )
    allow_chat_ids = resolve_allow_chat_ids(
        args,
        parse_allow_chat_ids_fn=parse_allow_chat_ids_fn,
    )
    if allow_chat_ids and int(chat_id) not in allow_chat_ids:
        raise ValueError(
            "Probe chat_id is not in allowlist. "
            f"chat_id={chat_id} allow_chat_ids={list(allow_chat_ids)}"
        )
    session_partition = normalize_session_partition_fn(getattr(args, "session_partition", None))

    return probe_config_cls(
        prompt=args.prompt,
        max_wait_secs=max_wait_secs,
        max_idle_secs=max_idle_secs,
        webhook_url=args.webhook_url,
        log_file=log_file,
        chat_id=int(chat_id),
        user_id=int(user_id),
        username=username,
        chat_title=(args.chat_title.strip() if args.chat_title else None),
        thread_id=thread_id,
        session_partition=session_partition,
        secret_token=(args.secret_token or telegram_webhook_secret_token_fn()),
        follow_logs=not args.no_follow,
        expect_events=tuple(args.expect_event),
        expect_reply_json_fields=expect_reply_json_fields,
        expect_log_regexes=tuple(args.expect_log_regex),
        expect_bot_regexes=tuple(args.expect_bot_regex),
        forbid_log_regexes=tuple(args.forbid_log_regex),
        fail_fast_error_logs=not args.no_fail_fast_error_log,
        allow_no_bot=bool(args.allow_no_bot),
        allow_chat_ids=allow_chat_ids,
        strong_update_id=True,
    )
