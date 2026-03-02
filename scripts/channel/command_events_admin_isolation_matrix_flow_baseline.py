#!/usr/bin/env python3
"""Baseline isolation phase for admin-isolation matrix assertions."""

from __future__ import annotations

from typing import Any

from command_events_admin_isolation_cases import build_admin_list_isolation_case


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
