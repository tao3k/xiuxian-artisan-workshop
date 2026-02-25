#!/usr/bin/env python3
"""Quality profile helpers for complex scenario probes."""

from __future__ import annotations

from typing import Any


def has_ancestor_with_tag(step_id: str, steps_by_id: dict[str, Any], tag: str) -> bool:
    """Return whether step has any dependency ancestor with the given tag."""
    target = tag.lower()
    visited: set[str] = set()
    stack = list(steps_by_id[step_id].depends_on)
    while stack:
        current = stack.pop()
        if current in visited:
            continue
        visited.add(current)
        current_step = steps_by_id.get(current)
        if current_step is None:
            continue
        if target in current_step.tags:
            return True
        stack.extend(current_step.depends_on)
    return False


def compute_quality_profile(
    scenario: Any, results: tuple[Any, ...], quality_profile_cls: Any
) -> Any:
    """Compute quality profile from step tags and runtime evidence."""
    steps_by_id = {step.step_id: step for step in scenario.steps}
    results_by_id = {result.step_id: result for result in results}

    error_signal_steps = [step for step in scenario.steps if "error_signal" in step.tags]
    correction_check_steps = [step for step in scenario.steps if "correction_check" in step.tags]
    natural_language_steps = [
        step for step in scenario.steps if not step.prompt.strip().startswith("/")
    ]

    negative_feedback_events = 0
    for step in error_signal_steps:
        result = results_by_id.get(step.step_id)
        if result is None:
            continue
        delta = result.feedback_command_bias_delta
        if isinstance(delta, (int, float)) and delta < 0:
            negative_feedback_events += 1

    successful_corrections = 0
    for step in correction_check_steps:
        result = results_by_id.get(step.step_id)
        if result is None or not result.passed:
            continue
        if not has_ancestor_with_tag(step.step_id, steps_by_id, "error_signal"):
            continue
        if not result.memory_planned_seen:
            continue
        successful_corrections += 1

    planned_hits = sum(1 for result in results if result.memory_planned_seen)
    recall_credit_events = sum(result.memory_recall_credit_count for result in results)
    decay_events = sum(result.memory_decay_count for result in results)

    quality_score = (
        len(error_signal_steps) * 2.0
        + negative_feedback_events * 3.0
        + len(correction_check_steps) * 2.0
        + successful_corrections * 4.0
        + planned_hits * 1.0
        + len(natural_language_steps) * 0.5
        + recall_credit_events * 0.5
        + decay_events * 1.0
    )

    return quality_profile_cls(
        error_signal_steps=len(error_signal_steps),
        negative_feedback_events=negative_feedback_events,
        correction_check_steps=len(correction_check_steps),
        successful_corrections=successful_corrections,
        planned_hits=planned_hits,
        natural_language_steps=len(natural_language_steps),
        recall_credit_events=recall_credit_events,
        decay_events=decay_events,
        quality_score=round(quality_score, 2),
    )
