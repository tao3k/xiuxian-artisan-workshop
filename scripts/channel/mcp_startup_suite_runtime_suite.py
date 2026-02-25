#!/usr/bin/env python3
"""Suite-level aggregation helpers for MCP startup runtime."""

from __future__ import annotations

import time
from datetime import UTC, datetime
from typing import Any

from mcp_startup_suite_runtime_mode_exec import run_mode


def run_suite(
    cfg: Any, *, build_mode_specs_fn: Any, evaluate_quality_gates_fn: Any
) -> dict[str, object]:
    """Execute startup suite and evaluate quality gate."""
    started_dt = datetime.now(UTC)
    started = time.monotonic()
    results = [run_mode(cfg, spec) for spec in build_mode_specs_fn(cfg)]
    passed_modes = sum(1 for mode in results if mode["passed"])
    failed_modes = len(results) - passed_modes
    quality_gate = evaluate_quality_gates_fn(cfg, results)
    overall_passed = failed_modes == 0 and bool(quality_gate.get("passed", False))
    return {
        "started_at": started_dt.isoformat(),
        "finished_at": datetime.now(UTC).isoformat(),
        "duration_ms": int((time.monotonic() - started) * 1000),
        "config": {
            "hot_rounds": cfg.hot_rounds,
            "hot_parallel": cfg.hot_parallel,
            "cold_rounds": cfg.cold_rounds,
            "cold_parallel": cfg.cold_parallel,
            "startup_timeout_secs": cfg.startup_timeout_secs,
            "cooldown_secs": cfg.cooldown_secs,
            "mcp_host": cfg.mcp_host,
            "mcp_port": cfg.mcp_port,
            "mcp_config": str(cfg.mcp_config),
            "health_url": cfg.health_url,
            "strict_health_check": cfg.strict_health_check,
            "health_probe_interval_secs": cfg.health_probe_interval_secs,
            "health_probe_timeout_secs": cfg.health_probe_timeout_secs,
            "restart_mcp_settle_secs": cfg.restart_mcp_settle_secs,
            "restart_health_timeout_secs": cfg.restart_health_timeout_secs,
            "restart_no_embedding": cfg.restart_no_embedding,
            "skip_hot": cfg.skip_hot,
            "skip_cold": cfg.skip_cold,
            "quality_max_failed_probes": cfg.quality_max_failed_probes,
            "quality_max_hot_p95_ms": cfg.quality_max_hot_p95_ms,
            "quality_max_cold_p95_ms": cfg.quality_max_cold_p95_ms,
            "quality_min_health_samples": cfg.quality_min_health_samples,
            "quality_max_health_failure_rate": cfg.quality_max_health_failure_rate,
            "quality_max_health_p95_ms": cfg.quality_max_health_p95_ms,
            "quality_baseline_json": str(cfg.quality_baseline_json)
            if cfg.quality_baseline_json
            else None,
            "quality_max_hot_p95_regression_ratio": cfg.quality_max_hot_p95_regression_ratio,
            "quality_max_cold_p95_regression_ratio": cfg.quality_max_cold_p95_regression_ratio,
        },
        "overall_passed": overall_passed,
        "passed_modes": passed_modes,
        "failed_modes": failed_modes,
        "quality_gate": quality_gate,
        "modes": results,
    }
