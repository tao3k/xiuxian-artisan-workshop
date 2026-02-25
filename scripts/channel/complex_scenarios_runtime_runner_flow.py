#!/usr/bin/env python3
"""Compatibility facade for complex scenario runtime orchestration flow."""

from __future__ import annotations

import time
from typing import Any

from complex_scenarios_runtime_runner_flow_complexity import (
    build_complexity_failed_result,
    evaluate_complexity_context,
)
from complex_scenarios_runtime_runner_flow_result import build_final_result
from complex_scenarios_runtime_runner_flow_waves import append_unreached_steps, execute_waves


def run_scenario(
    cfg: Any,
    scenario: Any,
    *,
    merge_requirements_fn: Any,
    merge_quality_requirements_fn: Any,
    compute_complexity_profile_fn: Any,
    evaluate_complexity_fn: Any,
    quality_profile_cls: Any,
    build_execution_waves_fn: Any,
    run_step_fn: Any,
    skipped_step_result_fn: Any,
    compute_quality_profile_fn: Any,
    evaluate_quality_fn: Any,
    scenario_run_result_cls: Any,
) -> Any:
    """Execute all scenario waves, then evaluate complexity + quality."""
    started = time.monotonic()

    context = evaluate_complexity_context(
        cfg,
        scenario,
        merge_requirements_fn=merge_requirements_fn,
        merge_quality_requirements_fn=merge_quality_requirements_fn,
        compute_complexity_profile_fn=compute_complexity_profile_fn,
        evaluate_complexity_fn=evaluate_complexity_fn,
    )
    if not context.complexity_passed:
        return build_complexity_failed_result(
            scenario,
            started_mono=started,
            context=context,
            quality_profile_cls=quality_profile_cls,
            scenario_run_result_cls=scenario_run_result_cls,
        )

    sessions = {session.alias: session for session in cfg.sessions}
    waves = build_execution_waves_fn(scenario)
    results = execute_waves(
        cfg,
        scenario,
        sessions=sessions,
        waves=waves,
        run_step_fn=run_step_fn,
        skipped_step_result_fn=skipped_step_result_fn,
    )
    results = append_unreached_steps(
        scenario,
        sessions=sessions,
        waves=waves,
        results=results,
        skipped_step_result_fn=skipped_step_result_fn,
    )
    return build_final_result(
        scenario,
        started_mono=started,
        context=context,
        results=results,
        compute_quality_profile_fn=compute_quality_profile_fn,
        evaluate_quality_fn=evaluate_quality_fn,
        scenario_run_result_cls=scenario_run_result_cls,
    )
