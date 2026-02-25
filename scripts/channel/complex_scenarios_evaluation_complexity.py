#!/usr/bin/env python3
"""Complexity profile helpers for complex scenario probes."""

from __future__ import annotations

from typing import Any


def build_execution_waves(scenario: Any) -> tuple[tuple[Any, ...], ...]:
    """Build topological execution waves from step dependencies."""
    pending: dict[str, Any] = {step.step_id: step for step in scenario.steps}
    completed: set[str] = set()
    waves: list[tuple[Any, ...]] = []

    while pending:
        ready = [
            step for step in pending.values() if all(dep in completed for dep in step.depends_on)
        ]
        if not ready:
            unresolved = sorted(pending)
            raise ValueError(
                f"scenario '{scenario.scenario_id}' has a dependency cycle or deadlock: {unresolved}"
            )
        ready.sort(key=lambda step: step.order)
        waves.append(tuple(ready))
        for step in ready:
            completed.add(step.step_id)
            pending.pop(step.step_id, None)

    return tuple(waves)


def compute_complexity_profile(scenario: Any, complexity_profile_cls: Any) -> Any:
    """Compute complexity profile from scenario graph shape."""
    waves = build_execution_waves(scenario)
    dependency_edges = sum(len(step.depends_on) for step in scenario.steps)

    children: dict[str, list[str]] = {step.step_id: [] for step in scenario.steps}
    longest_path: dict[str, int] = {}

    for step in scenario.steps:
        for dep in step.depends_on:
            children.setdefault(dep, []).append(step.step_id)

    for wave in waves:
        for step in wave:
            if not step.depends_on:
                longest_path[step.step_id] = 1
            else:
                longest_path[step.step_id] = max(longest_path[dep] for dep in step.depends_on) + 1

    critical_path_len = max(longest_path.values(), default=0)
    parallel_waves = sum(1 for wave in waves if len(wave) > 1)
    max_wave_width = max((len(wave) for wave in waves), default=0)
    branch_nodes = sum(1 for targets in children.values() if len(targets) > 1)

    complexity_score = (
        len(scenario.steps)
        + (dependency_edges * 1.5)
        + (critical_path_len * 2.0)
        + (parallel_waves * 3.0)
        + max_wave_width
        + branch_nodes
    )

    return complexity_profile_cls(
        step_count=len(scenario.steps),
        dependency_edges=dependency_edges,
        critical_path_len=critical_path_len,
        wave_count=len(waves),
        parallel_waves=parallel_waves,
        max_wave_width=max_wave_width,
        branch_nodes=branch_nodes,
        complexity_score=round(complexity_score, 2),
    )
