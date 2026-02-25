#!/usr/bin/env python3
"""Admin/isolation CLI argument group for command-events parser."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse


def add_admin_args(parser: argparse.ArgumentParser) -> None:
    """Add admin matrix/isolation command-events arguments."""
    parser.add_argument(
        "--admin-user-id",
        type=int,
        default=None,
        help=(
            "Optional Telegram user_id used for admin-only probes "
            "(`/reset`, `/resume drop`, `/session admin ...`). "
            "Falls back to $OMNI_TEST_ADMIN_USER_ID when omitted."
        ),
    )
    parser.add_argument(
        "--group-chat-id",
        type=int,
        default=None,
        help=(
            "Optional group chat_id for admin suite probes (`/session admin ...`). "
            "Falls back to $OMNI_TEST_GROUP_CHAT_ID when omitted."
        ),
    )
    parser.add_argument(
        "--admin-group-chat-id",
        action="append",
        type=int,
        default=[],
        help=("Repeatable admin-suite chat id for matrix runs. Used only with --admin-matrix."),
    )
    parser.add_argument(
        "--admin-matrix",
        action="store_true",
        help=(
            "Run selected admin cases across multiple group chats "
            "(from --admin-group-chat-id or group profile env file)."
        ),
    )
    parser.add_argument(
        "--matrix-retries",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MATRIX_RETRIES", "2")),
        help=(
            "Retry count for transient admin-matrix probe failures "
            "(default: 2, from $OMNI_BLACKBOX_MATRIX_RETRIES)."
        ),
    )
    parser.add_argument(
        "--matrix-backoff-secs",
        type=float,
        default=float(os.environ.get("OMNI_BLACKBOX_MATRIX_BACKOFF_SECS", "2")),
        help=(
            "Base backoff seconds for admin-matrix retries "
            "(default: 2, exponential: base*2^attempt)."
        ),
    )
    parser.add_argument(
        "--assert-admin-isolation",
        action="store_true",
        help=(
            "When used with --admin-matrix, run extra recipient-isolation checks: "
            "per-group add/list/clear plus cross-group zero-count assertions."
        ),
    )
    parser.add_argument(
        "--assert-admin-topic-isolation",
        action="store_true",
        help=(
            "Run extra same-group cross-topic isolation checks: delegated admins "
            "set in thread A must not leak into thread B."
        ),
    )
    parser.add_argument(
        "--group-thread-id",
        type=int,
        default=None,
        help=(
            "Optional topic thread id for admin suite probes. "
            "Falls back to $OMNI_TEST_GROUP_THREAD_ID when omitted."
        ),
    )
    parser.add_argument(
        "--group-thread-id-b",
        type=int,
        default=None,
        help=(
            "Optional secondary topic thread id for cross-topic isolation checks. "
            "Falls back to $OMNI_TEST_GROUP_THREAD_B; defaults to thread A + 1."
        ),
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("TELEGRAM_WEBHOOK_SECRET"),
        help=(
            "Webhook secret token passed through to agent_channel_blackbox.py. "
            "Defaults to $TELEGRAM_WEBHOOK_SECRET."
        ),
    )
