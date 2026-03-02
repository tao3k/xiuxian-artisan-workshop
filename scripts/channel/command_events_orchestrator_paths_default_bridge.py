#!/usr/bin/env python3
"""Default-mode bridge helpers for command-events orchestration paths."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_paths_default import run_default_mode as _run_default_mode_impl


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
