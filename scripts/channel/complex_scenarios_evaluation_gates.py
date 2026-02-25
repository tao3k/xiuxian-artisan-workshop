#!/usr/bin/env python3
"""Threshold gate evaluators for complex scenario probes."""

from __future__ import annotations

from typing import Any


def evaluate_complexity(profile: Any, requirement: Any) -> tuple[bool, tuple[str, ...]]:
    """Evaluate complexity profile against required thresholds."""
    failures: list[str] = []
    if profile.step_count < requirement.steps:
        failures.append(f"step_count={profile.step_count} < required={requirement.steps}")
    if profile.dependency_edges < requirement.dependency_edges:
        failures.append(
            f"dependency_edges={profile.dependency_edges} < required={requirement.dependency_edges}"
        )
    if profile.critical_path_len < requirement.critical_path_len:
        failures.append(
            "critical_path_len="
            f"{profile.critical_path_len} < required={requirement.critical_path_len}"
        )
    if profile.parallel_waves < requirement.parallel_waves:
        failures.append(
            f"parallel_waves={profile.parallel_waves} < required={requirement.parallel_waves}"
        )
    return (len(failures) == 0, tuple(failures))


def evaluate_quality(profile: Any, requirement: Any) -> tuple[bool, tuple[str, ...]]:
    """Evaluate quality profile against required thresholds."""
    failures: list[str] = []
    if profile.error_signal_steps < requirement.min_error_signals:
        failures.append(
            f"error_signal_steps={profile.error_signal_steps} < required={requirement.min_error_signals}"
        )
    if profile.negative_feedback_events < requirement.min_negative_feedback_events:
        failures.append(
            "negative_feedback_events="
            f"{profile.negative_feedback_events} < required={requirement.min_negative_feedback_events}"
        )
    if profile.correction_check_steps < requirement.min_correction_checks:
        failures.append(
            "correction_check_steps="
            f"{profile.correction_check_steps} < required={requirement.min_correction_checks}"
        )
    if profile.successful_corrections < requirement.min_successful_corrections:
        failures.append(
            "successful_corrections="
            f"{profile.successful_corrections} < required={requirement.min_successful_corrections}"
        )
    if profile.planned_hits < requirement.min_planned_hits:
        failures.append(
            f"planned_hits={profile.planned_hits} < required={requirement.min_planned_hits}"
        )
    if profile.natural_language_steps < requirement.min_natural_language_steps:
        failures.append(
            "natural_language_steps="
            f"{profile.natural_language_steps} < required={requirement.min_natural_language_steps}"
        )
    if profile.recall_credit_events < requirement.min_recall_credit_events:
        failures.append(
            "recall_credit_events="
            f"{profile.recall_credit_events} < required={requirement.min_recall_credit_events}"
        )
    if profile.decay_events < requirement.min_decay_events:
        failures.append(
            f"decay_events={profile.decay_events} < required={requirement.min_decay_events}"
        )
    return (len(failures) == 0, tuple(failures))
