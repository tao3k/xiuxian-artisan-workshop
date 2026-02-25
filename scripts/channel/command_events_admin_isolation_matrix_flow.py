#!/usr/bin/env python3
"""Execution flow helpers for admin-isolation matrix assertions."""

from __future__ import annotations

from typing import Any

from command_events_admin_isolation_cases import build_admin_list_isolation_case
from command_events_admin_isolation_common import run_isolation_case


def run_case(
    *,
    run_case_with_retry_fn: Any,
    blackbox_script: Any,
    case: Any,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    retries: int,
    backoff_secs: float,
    attempt_records: list[Any],
    mode_label: str,
    runtime_partition_mode: str | None,
) -> int:
    """Run one isolation case with shared matrix options."""
    return run_isolation_case(
        run_case_with_retry_fn=run_case_with_retry_fn,
        blackbox_script=blackbox_script,
        case=case,
        username=username,
        allow_chat_ids=allow_chat_ids,
        max_wait=max_wait,
        max_idle_secs=max_idle_secs,
        secret_token=secret_token,
        retries=retries,
        backoff_secs=backoff_secs,
        attempt_records=attempt_records,
        mode_label=mode_label,
        runtime_partition_mode=runtime_partition_mode,
    )


def run_baseline_isolation_cases(
    *,
    matrix_chat_ids: tuple[int, ...],
    admin_user_id: int | None,
    group_thread_id: int | None,
    build_cases_fn: Any,
    make_probe_case: Any,
    run_case_fn: Any,
) -> int:
    """Clear each matrix chat and assert zero admin overrides baseline."""
    for chat_id in matrix_chat_ids:
        clear_case = build_cases_fn(admin_user_id, chat_id, group_thread_id)
        clear_by_id = {case.case_id: case for case in clear_case}
        status = run_case_fn(
            case=clear_by_id["session_admin_clear"],
            mode_label="admin_matrix_isolation_baseline",
        )
        if status != 0:
            return status
        list_zero_case = build_admin_list_isolation_case(
            make_probe_case=make_probe_case,
            chat_id=chat_id,
            admin_user_id=admin_user_id,
            thread_id=group_thread_id,
            expected_override_count=0,
        )
        status = run_case_fn(
            case=list_zero_case,
            mode_label="admin_matrix_isolation_baseline",
        )
        if status != 0:
            return status
    return 0


def run_target_isolation_cases(
    *,
    matrix_chat_ids: tuple[int, ...],
    admin_user_id: int | None,
    group_thread_id: int | None,
    build_cases_fn: Any,
    make_probe_case: Any,
    run_case_fn: Any,
) -> int:
    """For each target chat, assert local add/list and cross-chat isolation."""
    for target_chat in matrix_chat_ids:
        scoped_cases = {
            case.case_id: case
            for case in build_cases_fn(admin_user_id, target_chat, group_thread_id)
        }
        status = run_case_fn(
            case=scoped_cases["session_admin_add"],
            mode_label="admin_matrix_isolation_target",
        )
        if status != 0:
            return status
        status = run_case_fn(
            case=build_admin_list_isolation_case(
                make_probe_case=make_probe_case,
                chat_id=target_chat,
                admin_user_id=admin_user_id,
                thread_id=group_thread_id,
                expected_override_count=1,
            ),
            mode_label="admin_matrix_isolation_target",
        )
        if status != 0:
            return status
        for other_chat in matrix_chat_ids:
            if other_chat == target_chat:
                continue
            status = run_case_fn(
                case=build_admin_list_isolation_case(
                    make_probe_case=make_probe_case,
                    chat_id=other_chat,
                    admin_user_id=admin_user_id,
                    thread_id=group_thread_id,
                    expected_override_count=0,
                ),
                mode_label="admin_matrix_isolation_cross_check",
            )
            if status != 0:
                return status
        status = run_case_fn(
            case=scoped_cases["session_admin_clear"],
            mode_label="admin_matrix_isolation_target",
        )
        if status != 0:
            return status
        status = run_case_fn(
            case=build_admin_list_isolation_case(
                make_probe_case=make_probe_case,
                chat_id=target_chat,
                admin_user_id=admin_user_id,
                thread_id=group_thread_id,
                expected_override_count=0,
            ),
            mode_label="admin_matrix_isolation_target",
        )
        if status != 0:
            return status
    return 0
