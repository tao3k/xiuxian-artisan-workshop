#!/usr/bin/env python3
"""Cross-chat matrix assertions for admin isolation probes."""

from __future__ import annotations

from functools import partial
from typing import Any

from command_events_admin_isolation_matrix_flow import (
    run_baseline_isolation_cases,
    run_case,
    run_target_isolation_cases,
)


def run_admin_isolation_assertions(
    *,
    blackbox_script: Any,
    matrix_chat_ids: tuple[int, ...],
    admin_user_id: int | None,
    group_thread_id: int | None,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempt_records: list[Any],
    runtime_partition_mode: str | None,
    build_cases_fn: Any,
    run_case_with_retry_fn: Any,
    make_probe_case: Any,
) -> int:
    """Run cross-chat admin isolation assertions for matrix mode."""
    print()
    print("=== Running admin recipient-isolation assertions ===")
    if len(matrix_chat_ids) < 2:
        print("Isolation assertions skipped: matrix has fewer than two chat ids.")
        return 0

    run_case_fn = partial(
        run_case,
        run_case_with_retry_fn=run_case_with_retry_fn,
        blackbox_script=blackbox_script,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempt_records=attempt_records,
        runtime_partition_mode=runtime_partition_mode,
    )
    status = run_baseline_isolation_cases(
        matrix_chat_ids=matrix_chat_ids,
        admin_user_id=admin_user_id,
        group_thread_id=group_thread_id,
        build_cases_fn=build_cases_fn,
        make_probe_case=make_probe_case,
        run_case_fn=run_case_fn,
    )
    if status != 0:
        return status
    return run_target_isolation_cases(
        matrix_chat_ids=matrix_chat_ids,
        admin_user_id=admin_user_id,
        group_thread_id=group_thread_id,
        build_cases_fn=build_cases_fn,
        make_probe_case=make_probe_case,
        run_case_fn=run_case_fn,
    )
