#!/usr/bin/env python3
"""Config-building helpers for dedup probe."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def _resolve_session_ids(
    *,
    chat_id: int | None,
    user_id: int | None,
    thread_id: int | None,
    log_file: Path,
    session_ids_from_runtime_log_fn: Any,
) -> tuple[int | None, int | None, int | None]:
    """Resolve missing chat/user/thread IDs from runtime logs."""
    if chat_id is None or user_id is None:
        inferred_chat, inferred_user, inferred_thread = session_ids_from_runtime_log_fn(log_file)
        if chat_id is None:
            chat_id = inferred_chat
        if user_id is None:
            user_id = inferred_user
        if thread_id is None:
            thread_id = inferred_thread
    return chat_id, user_id, thread_id


def _resolve_username(
    *,
    provided_username: str | None,
    log_file: Path,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
) -> str | None:
    """Resolve username with CLI > settings > runtime-log fallback."""
    username: str | None = provided_username.strip() if provided_username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)
    return username


def _resolve_secret_token(
    *, provided_secret: str | None, telegram_webhook_secret_token_fn: Any
) -> str | None:
    """Resolve webhook secret with CLI > resolver fallback."""
    secret_token: str | None = provided_secret.strip() if provided_secret else None
    if not secret_token:
        secret_token = telegram_webhook_secret_token_fn()
    return secret_token


def build_config(
    args: Any,
    *,
    probe_config_cls: Any,
    session_ids_from_runtime_log_fn: Any,
    username_from_settings_fn: Any,
    username_from_runtime_log_fn: Any,
    telegram_webhook_secret_token_fn: Any,
) -> Any:
    """Build validated probe config from parsed args."""
    chat_id: int | None = args.chat_id
    user_id: int | None = args.user_id
    thread_id: int | None = args.thread_id
    log_file = Path(args.log_file)

    chat_id, user_id, thread_id = _resolve_session_ids(
        chat_id=chat_id,
        user_id=user_id,
        thread_id=thread_id,
        log_file=log_file,
        session_ids_from_runtime_log_fn=session_ids_from_runtime_log_fn,
    )

    if chat_id is None or user_id is None:
        raise ValueError(
            "chat/user id are required. Use --chat-id/--user-id "
            "(or OMNI_TEST_CHAT_ID/OMNI_TEST_USER_ID). "
            "Tip: send one real Telegram message first so session_key can be inferred from logs."
        )
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be a positive integer.")

    username = _resolve_username(
        provided_username=args.username,
        log_file=log_file,
        username_from_settings_fn=username_from_settings_fn,
        username_from_runtime_log_fn=username_from_runtime_log_fn,
    )
    secret_token = _resolve_secret_token(
        provided_secret=args.secret_token,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token_fn,
    )

    return probe_config_cls(
        max_wait=args.max_wait,
        webhook_url=args.webhook_url,
        log_file=log_file,
        chat_id=int(chat_id),
        user_id=int(user_id),
        username=username,
        thread_id=thread_id,
        secret_token=secret_token,
        text=args.text,
    )
