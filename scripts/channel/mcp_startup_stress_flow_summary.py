#!/usr/bin/env python3
"""Summary helpers for MCP startup stress flow."""

from __future__ import annotations

import statistics
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Iterable


def summarize(
    results: Iterable[Any],
    health_samples: Iterable[Any],
    *,
    p95_fn: Any,
    summarize_health_samples_fn: Any,
) -> dict[str, object]:
    """Summarize probe results and health telemetry."""
    rows = list(results)
    total = len(rows)
    passed = sum(1 for row in rows if row.success)
    failed = total - passed
    reasons: dict[str, int] = {}
    for row in rows:
        reasons[row.reason] = reasons.get(row.reason, 0) + 1

    success_durations = [row.startup_duration_ms for row in rows if row.success]
    failure_durations = [row.startup_duration_ms for row in rows if not row.success]

    summary = {
        "total": total,
        "passed": passed,
        "failed": failed,
        "pass_rate": (passed / total) if total else 0.0,
        "reason_counts": reasons,
        "success_avg_startup_ms": statistics.fmean(success_durations) if success_durations else 0.0,
        "success_p95_startup_ms": p95_fn(success_durations),
        "failure_avg_startup_ms": statistics.fmean(failure_durations) if failure_durations else 0.0,
    }
    summary.update(summarize_health_samples_fn(health_samples))
    return summary
