#!/usr/bin/env python3
"""Compatibility facade for command event orchestration helpers."""

from __future__ import annotations

from typing import Any

from command_events_orchestrator_context import prepare_orchestrator_context
from command_events_orchestrator_execution import execute_probe_cases
from command_events_orchestrator_output import finalize_outputs


def run_command_events(
    args: Any,
    *,
    script_file: str,
    parse_optional_int_env_fn: Any,
    group_profile_int_fn: Any,
    resolve_allow_chat_ids_fn: Any,
    resolve_group_chat_id_fn: Any,
    resolve_topic_thread_pair_fn: Any,
    resolve_runtime_partition_mode_fn: Any,
    infer_group_thread_id_from_runtime_log_fn: Any,
    build_cases_fn: Any,
    select_cases_fn: Any,
    resolve_admin_matrix_chat_ids_fn: Any,
    run_case_with_retry_fn: Any,
    run_admin_isolation_assertions_fn: Any,
    run_admin_topic_isolation_assertions_fn: Any,
    build_report_fn: Any,
    write_outputs_fn: Any,
    telegram_webhook_secret_token_fn: Any,
    matrix_transient_exit_codes: set[int] | frozenset[int],
) -> int:
    """Execute command-events suite end-to-end from parsed args."""
    context, prepare_exit = prepare_orchestrator_context(
        args,
        script_file=script_file,
        parse_optional_int_env_fn=parse_optional_int_env_fn,
        group_profile_int_fn=group_profile_int_fn,
        resolve_allow_chat_ids_fn=resolve_allow_chat_ids_fn,
        resolve_group_chat_id_fn=resolve_group_chat_id_fn,
        resolve_topic_thread_pair_fn=resolve_topic_thread_pair_fn,
        resolve_runtime_partition_mode_fn=resolve_runtime_partition_mode_fn,
        infer_group_thread_id_from_runtime_log_fn=infer_group_thread_id_from_runtime_log_fn,
        telegram_webhook_secret_token_fn=telegram_webhook_secret_token_fn,
    )
    if prepare_exit is not None:
        return prepare_exit
    assert context is not None

    exit_code, matrix_chat_ids = execute_probe_cases(
        args,
        context=context,
        build_cases_fn=build_cases_fn,
        select_cases_fn=select_cases_fn,
        resolve_admin_matrix_chat_ids_fn=resolve_admin_matrix_chat_ids_fn,
        run_case_with_retry_fn=run_case_with_retry_fn,
        run_admin_isolation_assertions_fn=run_admin_isolation_assertions_fn,
        run_admin_topic_isolation_assertions_fn=run_admin_topic_isolation_assertions_fn,
        matrix_transient_exit_codes=matrix_transient_exit_codes,
    )
    if args.list_cases:
        return exit_code

    return finalize_outputs(
        args,
        context=context,
        matrix_chat_ids=matrix_chat_ids,
        exit_code=exit_code,
        build_report_fn=build_report_fn,
        write_outputs_fn=write_outputs_fn,
    )
