#!/usr/bin/env python3
"""Final result assembly helpers for complex scenario runner flow."""

from __future__ import annotations

import time
from typing import Any


def build_final_result(
    scenario: Any,
    *,
    started_mono: float,
    context: Any,
    results: list[Any],
    compute_quality_profile_fn: Any,
    evaluate_quality_fn: Any,
    scenario_run_result_cls: Any,
) -> Any:
    """Build final scenario run result from ordered step results."""
    duration_ms = int((time.monotonic() - started_mono) * 1000)
    passed = context.complexity_passed and all(step_result.passed for step_result in results)

    order = {step.step_id: step.order for step in scenario.steps}
    ordered_results = tuple(sorted(results, key=lambda result: order.get(result.step_id, 99999)))
    quality = compute_quality_profile_fn(scenario, ordered_results)
    quality_passed, quality_failures = evaluate_quality_fn(quality, context.quality_requirement)

    return scenario_run_result_cls(
        scenario_id=scenario.scenario_id,
        description=scenario.description,
        requirement=context.requirement,
        complexity=context.complexity,
        complexity_passed=context.complexity_passed,
        complexity_failures=context.complexity_failures,
        quality_requirement=context.quality_requirement,
        quality=quality,
        quality_passed=quality_passed,
        quality_failures=quality_failures,
        duration_ms=duration_ms,
        steps=ordered_results,
        passed=passed and quality_passed,
    )
