#!/usr/bin/env python3
"""Pipeline orchestration for acceptance runner."""

from __future__ import annotations

import time
from datetime import UTC, datetime
from typing import Any

from acceptance_runner_pipeline_execution import execute_step_with_retries, print_step_result
from acceptance_runner_pipeline_report import build_pipeline_report
from acceptance_runner_pipeline_steps import build_step_specs


def run_pipeline(
    cfg: Any,
    *,
    run_step_fn: Any,
    default_matrix_json: str,
    default_matrix_markdown: str,
    default_complex_json: str,
    default_complex_markdown: str,
    default_memory_json: str,
    default_memory_markdown: str,
    default_complex_dataset: str,
    default_memory_dataset: str,
    default_memory_scenario: str,
    python_executable: str,
) -> dict[str, object]:
    """Run acceptance pipeline and return structured report."""
    started = datetime.now(UTC)
    timer_start = time.perf_counter()
    results: list[Any] = []

    for spec in build_step_specs(
        cfg,
        default_matrix_json=default_matrix_json,
        default_matrix_markdown=default_matrix_markdown,
        default_complex_json=default_complex_json,
        default_complex_markdown=default_complex_markdown,
        default_memory_json=default_memory_json,
        default_memory_markdown=default_memory_markdown,
        default_complex_dataset=default_complex_dataset,
        default_memory_dataset=default_memory_dataset,
        default_memory_scenario=default_memory_scenario,
        python_executable=python_executable,
    ):
        result = execute_step_with_retries(
            spec,
            retries=cfg.retries,
            run_step_fn=run_step_fn,
        )
        results.append(result)
        print_step_result(result)
        if not result.passed:
            break

    return build_pipeline_report(
        cfg=cfg,
        steps=results,
        started_at=started,
        started_perf=timer_start,
        default_matrix_json=default_matrix_json,
        default_complex_json=default_complex_json,
        default_memory_json=default_memory_json,
        perf_counter_fn=time.perf_counter,
    )
