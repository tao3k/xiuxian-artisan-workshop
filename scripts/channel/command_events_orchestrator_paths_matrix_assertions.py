#!/usr/bin/env python3
"""Post-matrix assertion checks for command-events orchestration."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_paths_topic import run_admin_topic_isolation_if_requested


def run_matrix_assertions(
    *,
    exit_code: int,
    args: Any,
    admin_case_ids: list[str],
    matrix_chat_ids: tuple[int, ...],
    blackbox_script: Any,
    group_chat_id: int | None,
    group_thread_id: int | None,
    topic_thread_pair: tuple[int, int] | None,
    admin_user_id: int | None,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    attempts: list[Any],
    runtime_partition_mode: str | None,
    run_admin_isolation_assertions_fn: Any,
    run_admin_topic_isolation_assertions_fn: Any,
) -> int:
    """Run requested isolation assertions and return final exit code."""
    if exit_code != 0 or not admin_case_ids:
        return exit_code

    if args.assert_admin_isolation:
        isolation_status = run_admin_isolation_assertions_fn(
            blackbox_script=blackbox_script,
            matrix_chat_ids=matrix_chat_ids,
            admin_user_id=admin_user_id,
            group_thread_id=group_thread_id,
            username=username,
            allow_chat_ids=allow_chat_ids,
            max_wait=max_wait,
            max_idle_secs=max_idle_secs,
            secret_token=secret_token,
            retries=args.matrix_retries,
            backoff_secs=args.matrix_backoff_secs,
            attempt_records=attempts,
            runtime_partition_mode=runtime_partition_mode,
        )
        if isolation_status != 0:
            return isolation_status

    if args.assert_admin_topic_isolation:
        topic_status = run_admin_topic_isolation_if_requested(
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
            retries=args.matrix_retries,
            backoff_secs=args.matrix_backoff_secs,
            attempts=attempts,
            runtime_partition_mode=runtime_partition_mode,
        )
        if topic_status is not None and topic_status != 0:
            return topic_status

    return 0
