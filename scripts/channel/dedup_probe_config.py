#!/usr/bin/env python3
"""Argument parsing and config construction for dedup probe."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
from typing import Any


def parse_args(*, webhook_url_default: str) -> argparse.Namespace:
    """Parse CLI args for deterministic dedup probe."""
    parser = argparse.ArgumentParser(
        description=(
            "Post the same Telegram update_id twice to local webhook runtime and assert "
            "accepted/duplicate dedup events."
        )
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "25")),
        help="Max wait for dedup logs in seconds (default: 25).",
    )
    parser.add_argument(
        "--webhook-url",
        default=webhook_url_default,
        help="Webhook URL.",
    )
    parser.add_argument(
        "--log-file",
        default=os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log"),
        help="Runtime log file path.",
    )
    parser.add_argument(
        "--chat-id",
        type=int,
        default=int(os.environ["OMNI_TEST_CHAT_ID"]) if "OMNI_TEST_CHAT_ID" in os.environ else None,
        help="Telegram chat id (default: $OMNI_TEST_CHAT_ID).",
    )
    parser.add_argument(
        "--user-id",
        type=int,
        default=int(os.environ["OMNI_TEST_USER_ID"]) if "OMNI_TEST_USER_ID" in os.environ else None,
        help="Telegram user id (default: $OMNI_TEST_USER_ID).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME"),
        help="Telegram username (default: $OMNI_TEST_USERNAME).",
    )
    parser.add_argument(
        "--thread-id",
        type=int,
        default=int(os.environ["OMNI_TEST_THREAD_ID"])
        if "OMNI_TEST_THREAD_ID" in os.environ
        else None,
        help="Optional Telegram thread/topic id.",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
        help="Webhook secret header value.",
    )
    parser.add_argument(
        "--text",
        default="/session json",
        help="Message text payload (default: /session json).",
    )
    return parser.parse_args()


def build_config(
    args: argparse.Namespace,
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

    if chat_id is None or user_id is None:
        inferred_chat, inferred_user, inferred_thread = session_ids_from_runtime_log_fn(log_file)
        if chat_id is None:
            chat_id = inferred_chat
        if user_id is None:
            user_id = inferred_user
        if thread_id is None:
            thread_id = inferred_thread

    if chat_id is None or user_id is None:
        raise ValueError(
            "chat/user id are required. Use --chat-id/--user-id "
            "(or OMNI_TEST_CHAT_ID/OMNI_TEST_USER_ID). "
            "Tip: send one real Telegram message first so session_key can be inferred from logs."
        )
    if args.max_wait <= 0:
        raise ValueError("--max-wait must be a positive integer.")

    username: str | None = args.username.strip() if args.username else None
    if not username:
        username = username_from_settings_fn()
    if not username:
        username = username_from_runtime_log_fn(log_file)

    secret_token: str | None = args.secret_token.strip() if args.secret_token else None
    if not secret_token:
        secret_token = telegram_webhook_secret_token_fn()

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
