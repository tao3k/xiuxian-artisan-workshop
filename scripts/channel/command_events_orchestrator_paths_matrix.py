#!/usr/bin/env python3
"""Matrix-mode path for command-events orchestration."""

from __future__ import annotations

import sys
from typing import Any

from command_events_orchestrator_paths_matrix_admin import run_admin_matrix_cases
from command_events_orchestrator_paths_matrix_assertions import run_matrix_assertions
from command_events_orchestrator_paths_matrix_non_admin import run_non_admin_cases


def run_matrix_mode(
    *,
    args: Any,
    selected_cases: list[Any],
    group_chat_id: int | None,
    group_thread_id: int | None,
    topic_thread_pair: tuple[int, int] | None,
    admin_user_id: int | None,
    username: str,
    allow_chat_ids: tuple[str, ...],
    secret_token: str,
    blackbox_script: Any,
    runtime_partition_mode: str | None,
    attempts: list[Any],
    build_cases_fn: Any,
    run_case_with_retry_fn: Any,
    run_admin_isolation_assertions_fn: Any,
    run_admin_topic_isolation_assertions_fn: Any,
    resolve_admin_matrix_chat_ids_fn: Any,
    matrix_transient_exit_codes: set[int] | frozenset[int],
) -> tuple[int, tuple[int, ...]]:
    """Run matrix-mode execution path and return `(exit_code, matrix_chat_ids)`."""
    matrix_chat_ids = resolve_admin_matrix_chat_ids_fn(
        explicit_matrix_chat_ids=tuple(args.admin_group_chat_id),
        group_chat_id=group_chat_id,
        allow_chat_ids=allow_chat_ids,
    )
    if not matrix_chat_ids:
        print(
            "Error: --admin-matrix requested but no group chats resolved. "
            "Provide --admin-group-chat-id or set group profile/env chat ids.",
            file=sys.stderr,
        )
        return 2, ()

    matrix_chat_ids = tuple(matrix_chat_ids)
    exit_code, admin_case_ids = run_non_admin_cases(
        selected_cases=selected_cases,
        blackbox_script=blackbox_script,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=args.max_wait,
        max_idle_secs=args.max_idle_secs,
        secret_token=secret_token,
        attempts=attempts,
        runtime_partition_mode=runtime_partition_mode,
        run_case_with_retry_fn=run_case_with_retry_fn,
    )

    if exit_code == 0:
        exit_code = run_admin_matrix_cases(
            admin_case_ids=admin_case_ids,
            matrix_chat_ids=matrix_chat_ids,
            group_thread_id=group_thread_id,
            admin_user_id=admin_user_id,
            blackbox_script=blackbox_script,
            username=username,
            allow_chat_ids=allow_chat_ids,
            max_wait=args.max_wait,
            max_idle_secs=args.max_idle_secs,
            secret_token=secret_token,
            matrix_retries=args.matrix_retries,
            matrix_backoff_secs=args.matrix_backoff_secs,
            attempts=attempts,
            runtime_partition_mode=runtime_partition_mode,
            build_cases_fn=build_cases_fn,
            run_case_with_retry_fn=run_case_with_retry_fn,
            matrix_transient_exit_codes=matrix_transient_exit_codes,
        )

    exit_code = run_matrix_assertions(
        exit_code=exit_code,
        args=args,
        admin_case_ids=admin_case_ids,
        matrix_chat_ids=matrix_chat_ids,
        blackbox_script=blackbox_script,
        group_chat_id=group_chat_id,
        group_thread_id=group_thread_id,
        topic_thread_pair=topic_thread_pair,
        admin_user_id=admin_user_id,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=args.max_wait,
        max_idle_secs=args.max_idle_secs,
        secret_token=secret_token,
        attempts=attempts,
        runtime_partition_mode=runtime_partition_mode,
        run_admin_isolation_assertions_fn=run_admin_isolation_assertions_fn,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
    )

    return exit_code, matrix_chat_ids
