#!/usr/bin/env python3
"""Slow-response resilience gate checks for memory CI pipeline."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_quality_common import load_json, safe_int


def assert_evolution_slow_response_quality(cfg: Any) -> None:
    """Validate long-horizon slow-response resilience thresholds."""
    report = load_json(cfg.evolution_report_json)
    scenarios_obj = report.get("scenarios")
    scenarios = scenarios_obj if isinstance(scenarios_obj, list) else []
    if not scenarios:
        raise RuntimeError("evolution report has no scenarios for slow-response gate")

    scenario = scenarios[0] if isinstance(scenarios[0], dict) else {}
    duration_ms = safe_int(scenario.get("duration_ms"), default=0)
    steps_obj = scenario.get("steps")
    steps = steps_obj if isinstance(steps_obj, list) else []
    long_steps = sum(
        1
        for step in steps
        if isinstance(step, dict)
        and safe_int(step.get("duration_ms"), default=0) >= cfg.slow_response_long_step_ms
    )

    failures: list[str] = []
    if duration_ms < cfg.slow_response_min_duration_ms:
        failures.append(
            "evolution.duration_ms="
            f"{duration_ms} < slow_response_min_duration_ms={cfg.slow_response_min_duration_ms}"
        )
    if long_steps < cfg.slow_response_min_long_steps:
        failures.append(
            "evolution.long_steps="
            f"{long_steps} < slow_response_min_long_steps={cfg.slow_response_min_long_steps} "
            f"(threshold={cfg.slow_response_long_step_ms}ms)"
        )
    if failures:
        raise RuntimeError("slow-response resilience gate failed: " + "; ".join(failures))

    print(
        "Slow-response resilience gate passed: "
        f"duration_ms={duration_ms}, "
        f"long_steps={long_steps}, "
        f"long_step_threshold_ms={cfg.slow_response_long_step_ms}",
        flush=True,
    )
