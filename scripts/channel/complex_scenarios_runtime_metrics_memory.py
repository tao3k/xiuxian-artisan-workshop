#!/usr/bin/env python3
"""Memory-signal extraction helpers for complex scenario runtime probes."""

from __future__ import annotations

from typing import Any


def as_float(value: str) -> float | None:
    """Parse float token, returning None when parsing fails."""
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def extract_memory_metrics(
    stdout: str,
    *,
    ansi_escape_re: Any,
    memory_planned_bias_re: Any,
    memory_decision_re: Any,
    memory_feedback_re: Any,
    memory_recall_credit_re: Any,
    memory_decay_re: Any,
) -> dict[str, float | str | int | None]:
    """Extract memory signals from blackbox stdout."""
    normalized_lines = [ansi_escape_re.sub("", line) for line in stdout.splitlines()]
    planned_bias: float | None = None
    memory_decision: str | None = None
    recall_credit_count = 0
    decay_count = 0

    feedback_command_before: float | None = None
    feedback_command_after: float | None = None
    feedback_heuristic_before: float | None = None
    feedback_heuristic_after: float | None = None

    for line in normalized_lines:
        planned_match = memory_planned_bias_re.search(line)
        if planned_match:
            planned_bias = as_float(planned_match.group(1))

        decision_match = memory_decision_re.search(line)
        if decision_match:
            memory_decision = decision_match.group(1)
        if memory_recall_credit_re.search(line):
            recall_credit_count += 1
        if memory_decay_re.search(line):
            decay_count += 1

        feedback_match = memory_feedback_re.search(line)
        if feedback_match:
            source = feedback_match.group(1).strip()
            before = as_float(feedback_match.group(2))
            after = as_float(feedback_match.group(3))
            if source == "session_feedback_command":
                feedback_command_before = before
                feedback_command_after = after
            elif source == "assistant_heuristic":
                feedback_heuristic_before = before
                feedback_heuristic_after = after

    command_delta = None
    if feedback_command_before is not None and feedback_command_after is not None:
        command_delta = feedback_command_after - feedback_command_before

    heuristic_delta = None
    if feedback_heuristic_before is not None and feedback_heuristic_after is not None:
        heuristic_delta = feedback_heuristic_after - feedback_heuristic_before

    return {
        "memory_planned_bias": planned_bias,
        "memory_decision": memory_decision,
        "memory_recall_credit_count": recall_credit_count,
        "memory_decay_count": decay_count,
        "feedback_command_bias_before": feedback_command_before,
        "feedback_command_bias_after": feedback_command_after,
        "feedback_command_bias_delta": command_delta,
        "feedback_heuristic_bias_before": feedback_heuristic_before,
        "feedback_heuristic_bias_after": feedback_heuristic_after,
        "feedback_heuristic_bias_delta": heuristic_delta,
    }
