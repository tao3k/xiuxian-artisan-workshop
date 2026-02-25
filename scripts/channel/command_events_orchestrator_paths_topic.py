#!/usr/bin/env python3
"""Shared topic-isolation helpers for command-events orchestration paths."""

from __future__ import annotations

import sys
from typing import Any


def run_admin_topic_isolation_if_requested(
    *,
    args: Any,
    run_admin_topic_isolation_assertions_fn: Any,
    blackbox_script: Any,
    group_chat_id: int | None,
    topic_thread_pair: tuple[int, int] | None,
    admin_user_id: int | None,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempts: list[Any],
    runtime_partition_mode: str | None,
) -> int | None:
    """Run optional admin-topic isolation checks when requested."""
    if group_chat_id is None:
        print(
            "Error: --assert-admin-topic-isolation requires a resolved group chat id "
            "(--group-chat-id or OMNI_TEST_GROUP_CHAT_ID).",
            file=sys.stderr,
        )
        return 2
    if topic_thread_pair is None:
        print(
            "Error: --assert-admin-topic-isolation requires --group-thread-id "
            "(or OMNI_TEST_GROUP_THREAD_ID).",
            file=sys.stderr,
        )
        return 2

    thread_a, thread_b = topic_thread_pair
    return run_admin_topic_isolation_assertions_fn(
        blackbox_script=blackbox_script,
        group_chat_id=group_chat_id,
        admin_user_id=admin_user_id,
        thread_a=thread_a,
        thread_b=thread_b,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempt_records=attempts,
        runtime_partition_mode=runtime_partition_mode,
    )
