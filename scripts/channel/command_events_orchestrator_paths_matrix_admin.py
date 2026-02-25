#!/usr/bin/env python3
"""Admin-matrix case execution for command-events orchestration."""

from __future__ import annotations

from typing import Any


def run_admin_matrix_cases(
    *,
    admin_case_ids: list[str],
    matrix_chat_ids: tuple[int, ...],
    group_thread_id: int | None,
    admin_user_id: int | None,
    blackbox_script: Any,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    matrix_retries: int,
    matrix_backoff_secs: float,
    attempts: list[Any],
    runtime_partition_mode: str | None,
    build_cases_fn: Any,
    run_case_with_retry_fn: Any,
    matrix_transient_exit_codes: set[int] | frozenset[int],
) -> int:
    """Run admin cases across matrix chat targets and return exit code."""
    if not admin_case_ids:
        return 0

    print(
        "Selected admin matrix cases: "
        + ", ".join(admin_case_ids)
        + " | chats="
        + ",".join(str(chat_id) for chat_id in matrix_chat_ids)
    )
    print(
        "Admin matrix retry policy: "
        f"retries={matrix_retries} "
        f"transient_exit_codes={sorted(matrix_transient_exit_codes)} "
        f"base_backoff_secs={max(0.0, matrix_backoff_secs):.1f}"
    )

    for matrix_chat_id in matrix_chat_ids:
        print()
        print(f"=== Admin matrix target chat_id={matrix_chat_id} ===")
        scoped_cases_map = {
            case.case_id: case
            for case in build_cases_fn(
                admin_user_id,
                matrix_chat_id,
                group_thread_id,
            )
        }
        scoped_cases = [
            scoped_cases_map[case_id] for case_id in admin_case_ids if case_id in scoped_cases_map
        ]
        for case in scoped_cases:
            status = run_case_with_retry_fn(
                blackbox_script=blackbox_script,
                case=case,
                username=username,
                allow_chat_ids=allow_chat_ids,
                max_wait=max_wait,
                max_idle_secs=max_idle_secs,
                secret_token=secret_token,
                retries=matrix_retries,
                backoff_secs=matrix_backoff_secs,
                attempt_records=attempts,
                mode_label="admin_matrix",
                runtime_partition_mode=runtime_partition_mode,
            )
            if status != 0:
                return status

    return 0
