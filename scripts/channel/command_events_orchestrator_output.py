#!/usr/bin/env python3
"""Report rendering and output helpers for command event orchestration."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from command_events_orchestrator_context import OrchestratorContext


def finalize_outputs(
    args: Any,
    *,
    context: OrchestratorContext,
    matrix_chat_ids: tuple[int, ...],
    exit_code: int,
    build_report_fn: Any,
    write_outputs_fn: Any,
) -> int:
    """Build/write outputs and print concise final status."""
    report = build_report_fn(
        suites=context.suites,
        case_ids=tuple(args.case),
        allow_chat_ids=context.allow_chat_ids,
        matrix_chat_ids=matrix_chat_ids,
        attempts=context.attempts,
        started_dt=context.started_dt,
        started_mono=context.started_mono,
        exit_code=exit_code,
        runtime_partition_mode=context.runtime_partition_mode,
        admin_matrix=bool(args.admin_matrix),
        assert_admin_isolation=bool(args.assert_admin_isolation),
        assert_admin_topic_isolation=bool(args.assert_admin_topic_isolation),
        group_thread_id=args.group_thread_id,
        group_thread_id_b=args.group_thread_id_b,
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
        matrix_retries=int(args.matrix_retries),
        matrix_backoff_secs=float(args.matrix_backoff_secs),
    )
    write_outputs_fn(report, context.output_json, context.output_markdown)
    if exit_code == 0:
        print()
        print("All command event probes passed.")
    else:
        print()
        print(f"Command event probes failed with exit code {exit_code}.")
    print(f"JSON report: {context.output_json}")
    print(f"Markdown report: {context.output_markdown}")
    return exit_code
