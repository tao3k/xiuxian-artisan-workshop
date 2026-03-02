#!/usr/bin/env python3
"""Target/cross-check phase for admin-isolation matrix assertions."""

from __future__ import annotations

from typing import Any

from command_events_admin_isolation_cases import build_admin_list_isolation_case


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
