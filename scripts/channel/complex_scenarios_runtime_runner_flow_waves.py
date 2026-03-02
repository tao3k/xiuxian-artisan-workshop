#!/usr/bin/env python3
"""Wave execution helpers for complex scenario runner flow."""

from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from typing import Any


def execute_waves(
    cfg: Any,
    scenario: Any,
    *,
    sessions: dict[str, Any],
    waves: tuple[tuple[Any, ...], ...],
    run_step_fn: Any,
    skipped_step_result_fn: Any,
) -> list[Any]:
    """Execute scenario waves with dependency blocking and optional parallelism."""
    results: list[Any] = []
    failed: set[str] = set()

    for wave_index, wave in enumerate(waves):
        executable: list[Any] = []
        skipped: list[Any] = []

        for step in wave:
            blocked_deps = [dep for dep in step.depends_on if dep in failed]
            if blocked_deps:
                skipped.append(step)
            else:
                executable.append(step)

        for step in skipped:
            session = sessions[step.session_alias]
            reason = f"blocked_by_failed_dependencies={','.join(step.depends_on)}"
            result = skipped_step_result_fn(
                scenario.scenario_id,
                step,
                session,
                wave_index,
                reason,
            )
            results.append(result)
            failed.add(step.step_id)

        if not executable:
            continue

        if cfg.execute_wave_parallel and len(executable) > 1:
            workers = min(max(cfg.max_parallel, 1), len(executable))
            with ThreadPoolExecutor(max_workers=workers) as pool:
                futures = [
                    pool.submit(
                        run_step_fn,
                        cfg,
                        scenario.scenario_id,
                        step,
                        sessions[step.session_alias],
                        wave_index,
                    )
                    for step in executable
                ]
                wave_results = [future.result() for future in futures]
        else:
            wave_results = [
                run_step_fn(
                    cfg,
                    scenario.scenario_id,
                    step,
                    sessions[step.session_alias],
                    wave_index,
                )
                for step in executable
            ]

        for result in wave_results:
            results.append(result)
            if not result.passed:
                failed.add(result.step_id)

    return results


def append_unreached_steps(
    scenario: Any,
    *,
    sessions: dict[str, Any],
    waves: tuple[tuple[Any, ...], ...],
    results: list[Any],
    skipped_step_result_fn: Any,
) -> list[Any]:
    """Append synthetic skipped results for steps never reached after failures."""
    step_ids = {step.step_id for step in scenario.steps}
    already_recorded = {result.step_id for result in results}
    remaining = sorted(step_ids - already_recorded)
    if not remaining:
        return results

    steps_by_id = {step.step_id: step for step in scenario.steps}
    for step_id in remaining:
        step = steps_by_id[step_id]
        session = sessions[step.session_alias]
        results.append(
            skipped_step_result_fn(
                scenario.scenario_id,
                step,
                session,
                wave_index=len(waves),
                reason="not_reached_after_upstream_failure",
            )
        )
    return results
