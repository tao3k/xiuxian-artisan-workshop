#!/usr/bin/env python3
"""Non-admin case execution for command-events matrix mode."""

from __future__ import annotations

from typing import Any


def run_non_admin_cases(
    *,
    selected_cases: list[Any],
    blackbox_script: Any,
    username: str,
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    secret_token: str,
    attempts: list[Any],
    runtime_partition_mode: str | None,
    run_case_with_retry_fn: Any,
) -> tuple[int, list[str]]:
    """Run non-admin selected cases and return `(exit_code, admin_case_ids)`."""
    exit_code = 0
    non_admin_cases = [case for case in selected_cases if "admin" not in case.suites]
    admin_case_ids = [case.case_id for case in selected_cases if "admin" in case.suites]

    if non_admin_cases:
        print(
            "Selected non-admin cases: "
            + ", ".join(f"{case.case_id}[{','.join(case.suites)}]" for case in non_admin_cases)
        )
        for case in non_admin_cases:
            status = run_case_with_retry_fn(
                blackbox_script=blackbox_script,
                case=case,
                username=username,
                allow_chat_ids=allow_chat_ids,
                max_wait=max_wait,
                max_idle_secs=max_idle_secs,
                secret_token=secret_token,
                retries=0,
                backoff_secs=0,
                attempt_records=attempts,
                mode_label="matrix_non_admin",
                runtime_partition_mode=runtime_partition_mode,
            )
            if status != 0:
                exit_code = status
                break

    return exit_code, admin_case_ids
