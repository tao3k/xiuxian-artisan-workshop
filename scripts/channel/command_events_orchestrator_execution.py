#!/usr/bin/env python3
"""Execution helpers for command event probe orchestration."""

from __future__ import annotations

import sys
from typing import TYPE_CHECKING, Any

from command_events_orchestrator_paths import run_default_mode, run_matrix_mode

if TYPE_CHECKING:
    from command_events_orchestrator_context import OrchestratorContext


def execute_probe_cases(
    args: Any,
    *,
    context: OrchestratorContext,
    build_cases_fn: Any,
    select_cases_fn: Any,
    resolve_admin_matrix_chat_ids_fn: Any,
    run_case_with_retry_fn: Any,
    run_admin_isolation_assertions_fn: Any,
    run_admin_topic_isolation_assertions_fn: Any,
    matrix_transient_exit_codes: set[int] | frozenset[int],
) -> tuple[int, tuple[int, ...]]:
    """Select and execute probe cases in default or matrix mode."""
    all_cases = build_cases_fn(
        context.admin_user_id,
        context.group_chat_id,
        context.group_thread_id,
    )
    if args.list_cases:
        for case in all_cases:
            print(f"{case.case_id}\t[{','.join(case.suites)}]\t{case.prompt}")
        return 0, ()

    try:
        selected_cases = select_cases_fn(all_cases, context.suites, tuple(args.case))
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2, ()
    if not selected_cases:
        print("No cases selected.", file=sys.stderr)
        return 2, ()

    selected_admin_cases = [case for case in selected_cases if "admin" in case.suites]
    if args.admin_matrix:
        return run_matrix_mode(
            args=args,
            selected_cases=selected_cases,
            group_chat_id=context.group_chat_id,
            group_thread_id=context.group_thread_id,
            topic_thread_pair=context.topic_thread_pair,
            admin_user_id=context.admin_user_id,
            username=context.username,
            allow_chat_ids=context.allow_chat_ids,
            secret_token=context.secret_token,
            blackbox_script=context.blackbox_script,
            runtime_partition_mode=context.runtime_partition_mode,
            attempts=context.attempts,
            build_cases_fn=build_cases_fn,
            run_case_with_retry_fn=run_case_with_retry_fn,
            run_admin_isolation_assertions_fn=run_admin_isolation_assertions_fn,
            run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
            resolve_admin_matrix_chat_ids_fn=resolve_admin_matrix_chat_ids_fn,
            matrix_transient_exit_codes=matrix_transient_exit_codes,
        )

    exit_code = run_default_mode(
        args=args,
        selected_cases=selected_cases,
        selected_admin_cases=selected_admin_cases,
        group_chat_id=context.group_chat_id,
        topic_thread_pair=context.topic_thread_pair,
        admin_user_id=context.admin_user_id,
        username=context.username,
        allow_chat_ids=context.allow_chat_ids,
        secret_token=context.secret_token,
        blackbox_script=context.blackbox_script,
        runtime_partition_mode=context.runtime_partition_mode,
        attempts=context.attempts,
        run_case_with_retry_fn=run_case_with_retry_fn,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
    )
    return exit_code, ()
