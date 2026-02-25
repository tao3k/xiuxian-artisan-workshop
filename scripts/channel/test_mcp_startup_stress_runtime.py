#!/usr/bin/env python3
"""Unit tests for MCP startup stress runtime helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("mcp_startup_stress_runtime")


def test_classify_reason_priority() -> None:
    assert (
        module.classify_reason(
            ready_seen=True,
            handshake_timeout_seen=True,
            connect_failed_seen=True,
            process_exited=True,
            timed_out=True,
        )
        == "ok"
    )
    assert (
        module.classify_reason(
            ready_seen=False,
            handshake_timeout_seen=True,
            connect_failed_seen=False,
            process_exited=False,
            timed_out=False,
        )
        == "handshake_timeout"
    )


def test_summarize_health_samples_aggregates_counts() -> None:
    rows = [
        SimpleNamespace(ok=True, latency_ms=10.0, detail=""),
        SimpleNamespace(ok=False, latency_ms=0.0, detail="connection refused"),
        SimpleNamespace(ok=True, latency_ms=30.0, detail=""),
    ]

    summary = module.summarize_health_samples(rows)
    assert summary["health_samples_total"] == 3
    assert summary["health_samples_ok"] == 2
    assert summary["health_samples_failed"] == 1
    assert summary["health_failure_rate"] > 0
    assert summary["health_avg_latency_ms"] == 20.0
