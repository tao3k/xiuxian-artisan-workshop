#!/usr/bin/env python3
"""Matrix-mode bridge helpers for command-events orchestration paths."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_paths_matrix import run_matrix_mode as _run_matrix_mode_impl


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
    return _run_matrix_mode_impl(
        args=args,
        selected_cases=selected_cases,
        group_chat_id=group_chat_id,
        group_thread_id=group_thread_id,
        topic_thread_pair=topic_thread_pair,
        admin_user_id=admin_user_id,
        username=username,
        allow_chat_ids=allow_chat_ids,
        secret_token=secret_token,
        blackbox_script=blackbox_script,
        runtime_partition_mode=runtime_partition_mode,
        attempts=attempts,
        build_cases_fn=build_cases_fn,
        run_case_with_retry_fn=run_case_with_retry_fn,
        run_admin_isolation_assertions_fn=run_admin_isolation_assertions_fn,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
        resolve_admin_matrix_chat_ids_fn=resolve_admin_matrix_chat_ids_fn,
        matrix_transient_exit_codes=matrix_transient_exit_codes,
    )
