#!/usr/bin/env python3
"""Topic-isolation bridge helpers for command-events orchestration paths."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_paths_topic import (
    run_admin_topic_isolation_if_requested as _run_admin_topic_isolation_if_requested_impl,
)


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
    return _run_admin_topic_isolation_if_requested_impl(
        args=args,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
        blackbox_script=blackbox_script,
        group_chat_id=group_chat_id,
        topic_thread_pair=topic_thread_pair,
        admin_user_id=admin_user_id,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempts=attempts,
        runtime_partition_mode=runtime_partition_mode,
    )
