#!/usr/bin/env python3
"""Identity argument groups for session matrix parser."""

from __future__ import annotations

from typing import Any

import session_matrix_config_args_env as _env


def add_identity_args(parser: Any) -> None:
    """Add chat/user/thread identity arguments."""
    parser.add_argument(
        "--chat-id",
        type=int,
        default=_env.env_int("OMNI_TEST_CHAT_ID"),
        help="Telegram chat id (default: inferred from env/log).",
    )
    parser.add_argument(
        "--chat-b",
        type=int,
        default=_env.env_int("OMNI_TEST_CHAT_B"),
        help="Session B chat id (default: --chat-id or $OMNI_TEST_CHAT_B).",
    )
    parser.add_argument(
        "--chat-c",
        type=int,
        default=_env.env_int("OMNI_TEST_CHAT_C"),
        help="Session C chat id for mixed concurrency probe (default: --chat-id or $OMNI_TEST_CHAT_C).",
    )
    parser.add_argument(
        "--user-a",
        type=int,
        default=_env.env_int("OMNI_TEST_USER_ID"),
        help="Session A user id (default: inferred from env/log).",
    )
    parser.add_argument(
        "--user-b",
        type=int,
        default=_env.env_int("OMNI_TEST_USER_B"),
        help="Session B user id (default: user-a + 1 or $OMNI_TEST_USER_B).",
    )
    parser.add_argument(
        "--user-c",
        type=int,
        default=_env.env_int("OMNI_TEST_USER_C"),
        help="Session C user id for mixed concurrency probe (default: user-a + 2 or $OMNI_TEST_USER_C).",
    )
    parser.add_argument(
        "--username",
        default=_env.default_username(),
        help="Telegram username for allowlist checks.",
    )
    parser.add_argument(
        "--thread-a",
        type=int,
        default=_env.env_int("OMNI_TEST_THREAD_ID"),
        help="Optional thread id for session A.",
    )
    parser.add_argument(
        "--thread-b",
        type=int,
        default=_env.env_int("OMNI_TEST_THREAD_B"),
        help=(
            "Optional thread id for session B "
            "(default: thread-a + 1 when thread-a is provided, or $OMNI_TEST_THREAD_B)."
        ),
    )
    parser.add_argument(
        "--thread-c",
        type=int,
        default=_env.env_int("OMNI_TEST_THREAD_C"),
        help="Optional thread id for session C mixed probe (default: $OMNI_TEST_THREAD_C).",
    )
