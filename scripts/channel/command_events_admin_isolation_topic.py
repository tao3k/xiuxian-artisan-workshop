#!/usr/bin/env python3
"""Cross-topic assertions for admin isolation probes."""

from __future__ import annotations

from typing import Any

from command_events_admin_isolation_cases import build_admin_list_topic_isolation_case
from command_events_admin_isolation_common import run_isolation_case


def run_admin_topic_isolation_assertions(
    *,
    blackbox_script: Any,
    group_chat_id: int,
    admin_user_id: int | None,
    thread_a: int,
    thread_b: int,
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
    """Run cross-topic admin isolation assertions in one chat."""
    print()
    print(
        "=== Running admin topic-isolation assertions === "
        f"chat_id={group_chat_id} threads={thread_a},{thread_b}"
    )

    cases_a = {
        case.case_id: case for case in build_cases_fn(admin_user_id, group_chat_id, thread_a)
    }
    cases_b = {
        case.case_id: case for case in build_cases_fn(admin_user_id, group_chat_id, thread_b)
    }

    def _run(case: Any, mode_label: str) -> int:
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

    def _list_case(thread_id: int, expected_override_count: int) -> Any:
        return build_admin_list_topic_isolation_case(
            make_probe_case=make_probe_case,
            chat_id=group_chat_id,
            admin_user_id=admin_user_id,
            thread_id=thread_id,
            expected_override_count=expected_override_count,
        )

    for thread_id, clear_case, mode_label in (
        (thread_a, cases_a["session_admin_clear"], "admin_topic_isolation_baseline"),
        (thread_b, cases_b["session_admin_clear"], "admin_topic_isolation_baseline"),
    ):
        status = _run(clear_case, mode_label)
        if status != 0:
            return status
        status = _run(_list_case(thread_id, 0), mode_label)
        if status != 0:
            return status

    sequence: tuple[tuple[Any, str], ...] = (
        (cases_a["session_admin_add"], "admin_topic_isolation_target"),
        (_list_case(thread_a, 1), "admin_topic_isolation_target"),
        (_list_case(thread_b, 0), "admin_topic_isolation_cross_check"),
        (cases_a["session_admin_clear"], "admin_topic_isolation_target"),
        (_list_case(thread_a, 0), "admin_topic_isolation_target"),
        (cases_b["session_admin_add"], "admin_topic_isolation_target"),
        (_list_case(thread_b, 1), "admin_topic_isolation_target"),
        (_list_case(thread_a, 0), "admin_topic_isolation_cross_check"),
        (cases_b["session_admin_clear"], "admin_topic_isolation_target"),
        (_list_case(thread_b, 0), "admin_topic_isolation_target"),
    )
    for case, mode_label in sequence:
        status = _run(case, mode_label)
        if status != 0:
            return status

    return 0
