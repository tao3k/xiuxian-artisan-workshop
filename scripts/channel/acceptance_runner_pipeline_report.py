#!/usr/bin/env python3
"""Report assembly helpers for acceptance runner pipeline."""

from __future__ import annotations

from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


def build_pipeline_report(
    *,
    cfg: Any,
    steps: list[Any],
    started_at: datetime,
    started_perf: float,
    default_matrix_json: str,
    default_complex_json: str,
    default_memory_json: str,
    perf_counter_fn: Any,
) -> dict[str, object]:
    """Build final structured report for acceptance pipeline execution."""
    finished = datetime.now(UTC)
    duration_ms = int((perf_counter_fn() - started_perf) * 1000)
    passed_count = sum(1 for step in steps if step.passed)
    return {
        "started_at": started_at.isoformat(),
        "finished_at": finished.isoformat(),
        "duration_ms": duration_ms,
        "overall_passed": passed_count == len(steps),
        "summary": {
            "total": len(steps),
            "passed": passed_count,
            "failed": len(steps) - passed_count,
        },
        "config": asdict(cfg),
        "artifacts": {
            "group_profile_json": str(cfg.group_profile_json),
            "group_profile_env": str(cfg.group_profile_env),
            "matrix_json": default_matrix_json,
            "complex_json": default_complex_json,
            "memory_evolution_json": default_memory_json,
        },
        "steps": [asdict(step) for step in steps],
    }
