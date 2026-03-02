#!/usr/bin/env python3
"""Runner config assembly for complex scenarios runtime config."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def build_runner_config(
    args: Any,
    *,
    runner_config_cls: Any,
    complexity_requirement_cls: Any,
    quality_requirement_cls: Any,
    dataset_path: Path,
    blackbox_script: Path,
    log_file: Path,
    username: str | None,
    secret_token: str | None,
    runtime_partition_mode: str,
    sessions: tuple[Any, ...],
    forbid_log_regexes: tuple[str, ...],
) -> Any:
    """Build final typed runner config object."""
    return runner_config_cls(
        dataset_path=dataset_path,
        scenario_id=(args.scenario.strip() if args.scenario else None),
        blackbox_script=blackbox_script,
        webhook_url=args.webhook_url,
        log_file=log_file,
        username=username,
        secret_token=secret_token,
        max_wait=int(args.max_wait),
        max_idle_secs=int(args.max_idle_secs),
        max_parallel=int(args.max_parallel),
        execute_wave_parallel=bool(args.execute_wave_parallel),
        runtime_partition_mode=runtime_partition_mode,
        sessions=sessions,
        output_json=Path(args.output_json),
        output_markdown=Path(args.output_markdown),
        forbid_log_regexes=forbid_log_regexes,
        global_requirement=complexity_requirement_cls(
            steps=int(args.min_steps),
            dependency_edges=int(args.min_dependency_edges),
            critical_path_len=int(args.min_critical_path),
            parallel_waves=int(args.min_parallel_waves),
        ),
        global_quality_requirement=quality_requirement_cls(
            min_error_signals=int(args.min_error_signals),
            min_negative_feedback_events=int(args.min_negative_feedback_events),
            min_correction_checks=int(args.min_correction_checks),
            min_successful_corrections=int(args.min_successful_corrections),
            min_planned_hits=int(args.min_planned_hits),
            min_natural_language_steps=int(args.min_natural_language_steps),
            min_recall_credit_events=int(args.min_recall_credit_events),
            min_decay_events=int(args.min_decay_events),
        ),
    )
