#!/usr/bin/env python3
"""Quality threshold helpers for Discord ingress stress runtime summaries."""

from __future__ import annotations

from typing import Any


def evaluate_quality(cfg: Any, measured_rounds: list[Any]) -> tuple[bool, list[str]]:
    """Evaluate pass/fail against configured quality thresholds."""
    failures: list[str] = []
    total_requests = sum(int(row.total_requests) for row in measured_rounds)
    failed_requests = sum(int(row.failed_requests) for row in measured_rounds)
    failure_rate = (failed_requests / total_requests) if total_requests > 0 else 1.0

    if failure_rate > cfg.quality_max_failure_rate:
        failures.append(
            f"failure_rate {failure_rate:.4f} exceeded threshold {cfg.quality_max_failure_rate:.4f}"
        )

    if cfg.quality_max_p95_ms is not None:
        all_latencies = [float(row.p95_latency_ms) for row in measured_rounds]
        max_round_p95 = max(all_latencies) if all_latencies else 0.0
        if max_round_p95 > cfg.quality_max_p95_ms:
            failures.append(
                f"max_round_p95_ms {max_round_p95:.2f} exceeded "
                f"threshold {cfg.quality_max_p95_ms:.2f}"
            )

    if cfg.quality_min_rps is not None:
        average_rps = (
            sum(float(row.rps) for row in measured_rounds) / len(measured_rounds)
            if measured_rounds
            else 0.0
        )
        if average_rps < cfg.quality_min_rps:
            failures.append(
                f"average_rps {average_rps:.2f} below threshold {cfg.quality_min_rps:.2f}"
            )

    return (len(failures) == 0), failures
