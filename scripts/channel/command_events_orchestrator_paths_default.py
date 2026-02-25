#!/usr/bin/env python3
"""Default-mode path for command-events orchestration."""

from __future__ import annotations

import sys
from typing import Any

from command_events_orchestrator_paths_topic import run_admin_topic_isolation_if_requested


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
    exit_code = 0
    runnable_cases: list[Any] = []
    skipped_admin_cases: list[str] = []
    for case in selected_cases:
        if "admin" in case.suites and case.chat_id is None:
            skipped_admin_cases.append(case.case_id)
            continue
        runnable_cases.append(case)

    if skipped_admin_cases:
        print(
            "Skipping admin suite cases because no group chat id was provided "
            "(`--group-chat-id`, `OMNI_TEST_GROUP_CHAT_ID`, allowlist/group profile): "
            + ", ".join(skipped_admin_cases)
        )
    if not runnable_cases:
        print("No runnable cases left after filtering.", file=sys.stderr)
        return 2

    print(
        "Selected cases: "
        + ", ".join(f"{case.case_id}[{','.join(case.suites)}]" for case in runnable_cases)
    )

    for case in runnable_cases:
        status = run_case_with_retry_fn(
            blackbox_script=blackbox_script,
            case=case,
            username=username,
            allow_chat_ids=allow_chat_ids,
            max_wait=args.max_wait,
            max_idle_secs=args.max_idle_secs,
            secret_token=secret_token,
            retries=0,
            backoff_secs=0,
            attempt_records=attempts,
            mode_label="default",
            runtime_partition_mode=runtime_partition_mode,
        )
        if status != 0:
            exit_code = status
            break

    if exit_code == 0 and args.assert_admin_topic_isolation and selected_admin_cases:
        topic_status = run_admin_topic_isolation_if_requested(
            args=args,
            run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
            blackbox_script=blackbox_script,
            group_chat_id=group_chat_id,
            topic_thread_pair=topic_thread_pair,
            admin_user_id=admin_user_id,
            username=username,
            allow_chat_ids=allow_chat_ids,
            max_wait=args.max_wait,
            max_idle_secs=args.max_idle_secs,
            secret_token=secret_token,
            retries=args.matrix_retries,
            backoff_secs=args.matrix_backoff_secs,
            attempts=attempts,
            runtime_partition_mode=runtime_partition_mode,
        )
        if topic_status is not None and topic_status != 0:
            exit_code = topic_status

    return exit_code
