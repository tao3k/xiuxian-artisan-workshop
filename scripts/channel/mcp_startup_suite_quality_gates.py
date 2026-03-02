#!/usr/bin/env python3
"""Quality-gate evaluation logic for MCP startup suite runs."""

from __future__ import annotations

from typing import Any

from mcp_startup_suite_quality_baseline import (
    load_baseline_mode_p95s,
    mode_failed,
    mode_p95,
)


def evaluate_quality_gates(cfg: Any, modes: list[dict[str, object]]) -> dict[str, object]:
    """Evaluate quality gates for all executed startup modes."""
    violations: list[str] = []
    for mode in modes:
        summary = mode.get("summary")
        if not isinstance(summary, dict):
            violations.append(f"{mode['mode']}: missing mode summary")
            continue
        failed = mode_failed(summary)
        if failed > cfg.quality_max_failed_probes:
            violations.append(
                f"{mode['mode']}: failed probes {failed} > allowed {cfg.quality_max_failed_probes}"
            )

        p95 = mode_p95(summary)
        if mode["mode"] == "hot" and p95 > cfg.quality_max_hot_p95_ms:
            violations.append(
                f"hot: p95 {p95:.1f}ms > threshold {cfg.quality_max_hot_p95_ms:.1f}ms"
            )
        if mode["mode"] == "cold" and p95 > cfg.quality_max_cold_p95_ms:
            violations.append(
                f"cold: p95 {p95:.1f}ms > threshold {cfg.quality_max_cold_p95_ms:.1f}ms"
            )
        health_samples = int(summary.get("health_samples_total", 0))
        health_failure_rate = float(summary.get("health_failure_rate", 0.0))
        health_p95 = float(summary.get("health_p95_latency_ms", 0.0))
        if health_samples < cfg.quality_min_health_samples:
            violations.append(
                f"{mode['mode']}: health samples {health_samples} < required "
                f"{cfg.quality_min_health_samples}"
            )
        if health_failure_rate > cfg.quality_max_health_failure_rate:
            violations.append(
                f"{mode['mode']}: health failure rate {health_failure_rate:.2%} > threshold "
                f"{cfg.quality_max_health_failure_rate:.2%}"
            )
        if health_p95 > cfg.quality_max_health_p95_ms:
            violations.append(
                f"{mode['mode']}: health p95 {health_p95:.1f}ms > threshold "
                f"{cfg.quality_max_health_p95_ms:.1f}ms"
            )

    baseline_p95: dict[str, float] = {}
    if cfg.quality_baseline_json is not None:
        baseline_p95 = load_baseline_mode_p95s(cfg.quality_baseline_json)
        for mode in modes:
            summary = mode.get("summary")
            if not isinstance(summary, dict):
                continue
            mode_name = str(mode.get("mode"))
            current_p95 = mode_p95(summary)
            base_p95 = baseline_p95.get(mode_name)
            if base_p95 is None or base_p95 <= 0:
                continue
            ratio = (current_p95 - base_p95) / base_p95
            if mode_name == "hot" and ratio > cfg.quality_max_hot_p95_regression_ratio:
                violations.append(
                    f"hot: p95 regression {ratio:.2%} > allowed "
                    f"{cfg.quality_max_hot_p95_regression_ratio:.2%} "
                    f"(baseline={base_p95:.1f}ms current={current_p95:.1f}ms)"
                )
            if mode_name == "cold" and ratio > cfg.quality_max_cold_p95_regression_ratio:
                violations.append(
                    f"cold: p95 regression {ratio:.2%} > allowed "
                    f"{cfg.quality_max_cold_p95_regression_ratio:.2%} "
                    f"(baseline={base_p95:.1f}ms current={current_p95:.1f}ms)"
                )

    return {
        "passed": len(violations) == 0,
        "violations": violations,
        "thresholds": {
            "max_failed_probes": cfg.quality_max_failed_probes,
            "max_hot_p95_ms": cfg.quality_max_hot_p95_ms,
            "max_cold_p95_ms": cfg.quality_max_cold_p95_ms,
            "min_health_samples": cfg.quality_min_health_samples,
            "max_health_failure_rate": cfg.quality_max_health_failure_rate,
            "max_health_p95_ms": cfg.quality_max_health_p95_ms,
            "max_hot_p95_regression_ratio": cfg.quality_max_hot_p95_regression_ratio,
            "max_cold_p95_regression_ratio": cfg.quality_max_cold_p95_regression_ratio,
        },
        "baseline_json": str(cfg.quality_baseline_json) if cfg.quality_baseline_json else None,
        "baseline_mode_p95_ms": baseline_p95,
    }
