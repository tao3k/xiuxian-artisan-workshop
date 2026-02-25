#!/usr/bin/env python3
"""Session identity parser sections for complex scenarios config."""

from __future__ import annotations

from typing import Any


def add_session_identity_args(parser: Any, *, env_int_fn: Any) -> None:
    """Register session chat/user/thread identity args."""
    parser.add_argument(
        "--chat-a",
        type=int,
        default=env_int_fn("OMNI_TEST_CHAT_ID"),
        help="Session A chat id (default: $OMNI_TEST_CHAT_ID).",
    )
    parser.add_argument(
        "--chat-b",
        type=int,
        default=env_int_fn("OMNI_TEST_CHAT_B"),
        help="Session B chat id (default: $OMNI_TEST_CHAT_B).",
    )
    parser.add_argument(
        "--chat-c",
        type=int,
        default=env_int_fn("OMNI_TEST_CHAT_C"),
        help="Session C chat id (default: $OMNI_TEST_CHAT_C).",
    )
    parser.add_argument(
        "--user-a",
        type=int,
        default=env_int_fn("OMNI_TEST_USER_ID"),
        help="Session A user id (default: $OMNI_TEST_USER_ID).",
    )
    parser.add_argument(
        "--user-b",
        type=int,
        default=env_int_fn("OMNI_TEST_USER_B"),
        help="Session B user id (default: $OMNI_TEST_USER_B).",
    )
    parser.add_argument(
        "--user-c",
        type=int,
        default=env_int_fn("OMNI_TEST_USER_C"),
        help="Session C user id (default: $OMNI_TEST_USER_C).",
    )
    parser.add_argument(
        "--thread-a",
        type=int,
        default=env_int_fn("OMNI_TEST_THREAD_ID"),
        help="Session A thread id (default: $OMNI_TEST_THREAD_ID).",
    )
    parser.add_argument(
        "--thread-b",
        type=int,
        default=env_int_fn("OMNI_TEST_THREAD_B"),
        help="Session B thread id (default: $OMNI_TEST_THREAD_B).",
    )
    parser.add_argument(
        "--thread-c",
        type=int,
        default=env_int_fn("OMNI_TEST_THREAD_C"),
        help="Session C thread id (default: $OMNI_TEST_THREAD_C).",
    )
    parser.add_argument("--chat-title-a", default=None, help="Synthetic chat title for session A.")
    parser.add_argument("--chat-title-b", default=None, help="Synthetic chat title for session B.")
    parser.add_argument("--chat-title-c", default=None, help="Synthetic chat title for session C.")
