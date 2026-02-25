#!/usr/bin/env python3
"""Requirement parsing/merge helpers for complex scenario datasets."""

from __future__ import annotations

from typing import Any


def required_str_field(obj: dict[str, object], key: str, *, ctx: str) -> str:
    """Read a required non-empty string field from an object."""
    value = str(obj.get(key, "")).strip()
    if not value:
        raise ValueError(f"{ctx}: missing non-empty '{key}'")
    return value


def parse_requirement(raw: dict[str, object] | None, *, requirement_cls: Any) -> Any:
    """Parse complexity requirement payload."""
    if raw is None:
        return None
    return requirement_cls(
        steps=int(raw.get("steps", 0)),
        dependency_edges=int(raw.get("dependency_edges", 0)),
        critical_path_len=int(raw.get("critical_path_len", 0)),
        parallel_waves=int(raw.get("parallel_waves", 0)),
    )


def parse_quality_requirement(
    raw: dict[str, object] | None, *, quality_requirement_cls: Any
) -> Any:
    """Parse quality requirement payload."""
    if raw is None:
        return None
    return quality_requirement_cls(
        min_error_signals=int(raw.get("min_error_signals", 0)),
        min_negative_feedback_events=int(raw.get("min_negative_feedback_events", 0)),
        min_correction_checks=int(raw.get("min_correction_checks", 0)),
        min_successful_corrections=int(raw.get("min_successful_corrections", 0)),
        min_planned_hits=int(raw.get("min_planned_hits", 0)),
        min_natural_language_steps=int(raw.get("min_natural_language_steps", 0)),
        min_recall_credit_events=int(raw.get("min_recall_credit_events", 0)),
        min_decay_events=int(raw.get("min_decay_events", 0)),
    )


def merge_requirements(
    global_requirement: Any, scenario_requirement: Any, *, requirement_cls: Any
) -> Any:
    """Merge global + scenario-specific complexity requirements."""
    if scenario_requirement is None:
        return global_requirement
    return requirement_cls(
        steps=max(global_requirement.steps, scenario_requirement.steps),
        dependency_edges=max(
            global_requirement.dependency_edges,
            scenario_requirement.dependency_edges,
        ),
        critical_path_len=max(
            global_requirement.critical_path_len,
            scenario_requirement.critical_path_len,
        ),
        parallel_waves=max(
            global_requirement.parallel_waves,
            scenario_requirement.parallel_waves,
        ),
    )


def merge_quality_requirements(
    global_requirement: Any,
    scenario_requirement: Any,
    *,
    quality_requirement_cls: Any,
) -> Any:
    """Merge global + scenario-specific quality requirements."""
    if scenario_requirement is None:
        return global_requirement
    return quality_requirement_cls(
        min_error_signals=max(
            global_requirement.min_error_signals,
            scenario_requirement.min_error_signals,
        ),
        min_negative_feedback_events=max(
            global_requirement.min_negative_feedback_events,
            scenario_requirement.min_negative_feedback_events,
        ),
        min_correction_checks=max(
            global_requirement.min_correction_checks,
            scenario_requirement.min_correction_checks,
        ),
        min_successful_corrections=max(
            global_requirement.min_successful_corrections,
            scenario_requirement.min_successful_corrections,
        ),
        min_planned_hits=max(
            global_requirement.min_planned_hits,
            scenario_requirement.min_planned_hits,
        ),
        min_natural_language_steps=max(
            global_requirement.min_natural_language_steps,
            scenario_requirement.min_natural_language_steps,
        ),
        min_recall_credit_events=max(
            global_requirement.min_recall_credit_events,
            scenario_requirement.min_recall_credit_events,
        ),
        min_decay_events=max(
            global_requirement.min_decay_events,
            scenario_requirement.min_decay_events,
        ),
    )
