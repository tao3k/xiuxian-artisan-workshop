#!/usr/bin/env python3
"""Complexity-evaluation helpers for complex scenario runner flow."""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class ComplexityContext:
    """Merged requirements and computed complexity evaluation for one scenario."""

    requirement: Any
    quality_requirement: Any
    complexity: Any
    complexity_passed: bool
    complexity_failures: tuple[str, ...]


def evaluate_complexity_context(
    cfg: Any,
    scenario: Any,
    *,
    merge_requirements_fn: Any,
    merge_quality_requirements_fn: Any,
    compute_complexity_profile_fn: Any,
    evaluate_complexity_fn: Any,
) -> ComplexityContext:
    """Build merged requirements and evaluate complexity profile."""
    requirement = merge_requirements_fn(cfg.global_requirement, scenario.required_complexity)
    quality_requirement = merge_quality_requirements_fn(
        cfg.global_quality_requirement,
        scenario.required_quality,
    )
    complexity = compute_complexity_profile_fn(scenario)
    complexity_passed, complexity_failures = evaluate_complexity_fn(complexity, requirement)
    return ComplexityContext(
        requirement=requirement,
        quality_requirement=quality_requirement,
        complexity=complexity,
        complexity_passed=complexity_passed,
        complexity_failures=complexity_failures,
    )


def build_complexity_failed_result(
    scenario: Any,
    *,
    started_mono: float,
    context: ComplexityContext,
    quality_profile_cls: Any,
    scenario_run_result_cls: Any,
) -> Any:
    """Build final scenario result for complexity-failure short-circuit."""
    duration_ms = int((time.monotonic() - started_mono) * 1000)
    quality = quality_profile_cls(
        error_signal_steps=0,
        negative_feedback_events=0,
        correction_check_steps=0,
        successful_corrections=0,
        planned_hits=0,
        natural_language_steps=0,
        recall_credit_events=0,
        decay_events=0,
        quality_score=0.0,
    )
    return scenario_run_result_cls(
        scenario_id=scenario.scenario_id,
        description=scenario.description,
        requirement=context.requirement,
        complexity=context.complexity,
        complexity_passed=False,
        complexity_failures=context.complexity_failures,
        quality_requirement=context.quality_requirement,
        quality=quality,
        quality_passed=False,
        quality_failures=("quality_skipped_due_to_complexity_failure",),
        duration_ms=duration_ms,
        steps=(),
        passed=False,
    )
