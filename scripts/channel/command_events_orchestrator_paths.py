#!/usr/bin/env python3
"""Execution-path helpers for command-events orchestration."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_paths_default import run_default_mode as _run_default_mode_impl
from command_events_orchestrator_paths_matrix import run_matrix_mode as _run_matrix_mode_impl
from command_events_orchestrator_paths_topic import (
    run_admin_topic_isolation_if_requested as _run_admin_topic_isolation_if_requested_impl,
)


def _run_admin_topic_isolation_if_requested(
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


def run_default_mode(
    *,
    args: Any,
    selected_cases: list[Any],
    selected_admin_cases: list[Any],
    group_chat_id: int | None,
    topic_thread_pair: tuple[int, int] | None,
    admin_user_id: int | None,
    username: str,
    allow_chat_ids: tuple[str, ...],
    secret_token: str,
    blackbox_script: Any,
    runtime_partition_mode: str | None,
    attempts: list[Any],
    run_case_with_retry_fn: Any,
    run_admin_topic_isolation_assertions_fn: Any,
) -> int:
    """Run non-matrix execution path and return exit code."""
    return _run_default_mode_impl(
        args=args,
        selected_cases=selected_cases,
        selected_admin_cases=selected_admin_cases,
        group_chat_id=group_chat_id,
        topic_thread_pair=topic_thread_pair,
        admin_user_id=admin_user_id,
        username=username,
        allow_chat_ids=allow_chat_ids,
        secret_token=secret_token,
        blackbox_script=blackbox_script,
        runtime_partition_mode=runtime_partition_mode,
        attempts=attempts,
        run_case_with_retry_fn=run_case_with_retry_fn,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
    )
