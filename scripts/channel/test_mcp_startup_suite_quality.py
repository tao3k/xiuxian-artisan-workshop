#!/usr/bin/env python3
"""Unit tests for MCP startup suite quality helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("mcp_startup_suite_quality")


def test_evaluate_quality_gates_accepts_good_mode() -> None:
    cfg = SimpleNamespace(
        quality_max_failed_probes=0,
        quality_max_hot_p95_ms=1000.0,
        quality_max_cold_p95_ms=1000.0,
        quality_min_health_samples=1,
        quality_max_health_failure_rate=0.2,
        quality_max_health_p95_ms=500.0,
        quality_baseline_json=None,
        quality_max_hot_p95_regression_ratio=0.5,
        quality_max_cold_p95_regression_ratio=0.5,
    )
    modes = [
        {
            "mode": "hot",
            "summary": {
                "failed": 0,
                "success_p95_startup_ms": 100.0,
                "health_samples_total": 10,
                "health_failure_rate": 0.0,
                "health_p95_latency_ms": 50.0,
            },
        }
    ]

    result = module.evaluate_quality_gates(cfg, modes)
    assert result["passed"] is True
    assert result["violations"] == []
