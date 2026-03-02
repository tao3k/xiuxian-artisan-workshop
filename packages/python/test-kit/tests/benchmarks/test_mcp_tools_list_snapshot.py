"""Unit tests for MCP tools/list benchmark YAML snapshot helpers."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from omni.test_kit.mcp_tools_list_snapshot import (
    build_mcp_tools_list_snapshot_payload,
    detect_mcp_tools_list_snapshot_anomalies,
    load_mcp_tools_list_snapshot,
    save_mcp_tools_list_snapshot,
)

if TYPE_CHECKING:
    from pathlib import Path


def _summary(
    *,
    p95_ms: float,
    p99_ms: float,
    concurrency: int = 40,
    base_url: str = "http://127.0.0.1:3002",
) -> dict[str, object]:
    return {
        "base_url": base_url,
        "slo": {"p95_ms": 400.0, "p99_ms": 800.0},
        "total_per_point": 1000,
        "concurrency_values": [concurrency],
        "summary": {"mean_rps": 1200.0, "error_total": 0},
        "recommendation": {
            "recommended_concurrency": concurrency,
            "knee_concurrency": 80,
            "reason": "selected highest-RPS point within SLO bounds",
        },
        "points": [
            {
                "concurrency": concurrency,
                "total": 1000,
                "errors": 0,
                "elapsed_s": 1.0,
                "rps": 1000.0,
                "p50_ms": 50.0,
                "p95_ms": p95_ms,
                "p99_ms": p99_ms,
            }
        ],
    }


def test_build_snapshot_payload_smooths_baselines_and_preserves_overrides() -> None:
    previous = {
        "targets": {
            "http://127.0.0.1:3002": {
                "points": {
                    "40": {
                        "baseline_p95_ms": 200.0,
                        "baseline_p99_ms": 300.0,
                        "regression_factor": 2.5,
                        "min_regression_delta_ms": 90.0,
                    }
                }
            }
        }
    }
    payload = build_mcp_tools_list_snapshot_payload(
        summary=_summary(p95_ms=100.0, p99_ms=150.0),
        previous=previous,
        alpha=0.25,
    )

    target = payload["targets"]["http://127.0.0.1:3002"]
    point = target["points"]["40"]
    assert point["baseline_p95_ms"] == pytest.approx(175.0)
    assert point["baseline_p99_ms"] == pytest.approx(262.5)
    assert point["last_p95_ms"] == pytest.approx(100.0)
    assert point["last_p99_ms"] == pytest.approx(150.0)
    assert point["regression_factor"] == pytest.approx(2.5)
    assert point["min_regression_delta_ms"] == pytest.approx(90.0)


def test_detect_snapshot_anomalies_uses_default_and_per_point_thresholds() -> None:
    snapshot = {
        "defaults": {"regression_factor": 2.0, "min_regression_delta_ms": 40.0},
        "targets": {
            "http://127.0.0.1:3002": {
                "points": {
                    "40": {"baseline_p95_ms": 100.0, "baseline_p99_ms": 150.0},
                    "80": {
                        "baseline_p95_ms": 200.0,
                        "baseline_p99_ms": 320.0,
                        "regression_factor": 3.0,
                    },
                }
            }
        },
    }
    summary = {
        **_summary(p95_ms=260.0, p99_ms=260.0, concurrency=40),
        "points": [
            {
                "concurrency": 40,
                "total": 1000,
                "errors": 0,
                "elapsed_s": 1.0,
                "rps": 1000.0,
                "p50_ms": 60.0,
                "p95_ms": 260.0,
                "p99_ms": 260.0,
            },
            {
                "concurrency": 80,
                "total": 1000,
                "errors": 0,
                "elapsed_s": 1.0,
                "rps": 900.0,
                "p50_ms": 80.0,
                "p95_ms": 500.0,
                "p99_ms": 500.0,
            },
        ],
    }

    anomalies = detect_mcp_tools_list_snapshot_anomalies(summary=summary, snapshot=snapshot)
    assert len(anomalies) == 1
    assert anomalies[0].concurrency == 40
    assert anomalies[0].metric == "p95_ms"
    assert anomalies[0].threshold_ms == pytest.approx(200.0)


def test_detect_snapshot_anomalies_skips_points_outside_snapshot_slo_budget() -> None:
    snapshot = {
        "benchmark": {"p95_slo_ms": 400.0, "p99_slo_ms": 800.0},
        "defaults": {"regression_factor": 2.0, "min_regression_delta_ms": 40.0},
        "targets": {
            "http://127.0.0.1:3002": {
                "points": {
                    "40": {"baseline_p95_ms": 120.0, "baseline_p99_ms": 180.0},
                    "120": {"baseline_p95_ms": 500.0, "baseline_p99_ms": 700.0},
                }
            }
        },
    }
    summary = {
        **_summary(p95_ms=160.0, p99_ms=210.0, concurrency=40),
        "points": [
            {
                "concurrency": 40,
                "total": 1000,
                "errors": 0,
                "elapsed_s": 1.0,
                "rps": 1000.0,
                "p50_ms": 60.0,
                "p95_ms": 160.0,
                "p99_ms": 210.0,
            },
            {
                "concurrency": 120,
                "total": 1000,
                "errors": 0,
                "elapsed_s": 1.0,
                "rps": 600.0,
                "p50_ms": 220.0,
                "p95_ms": 1200.0,
                "p99_ms": 1800.0,
            },
        ],
    }

    anomalies = detect_mcp_tools_list_snapshot_anomalies(summary=summary, snapshot=snapshot)
    assert anomalies == []


def test_save_and_load_snapshot_roundtrip(tmp_path: Path) -> None:
    path = tmp_path / "mcp_tools_list.yaml"
    payload = {
        "schema": "omni.agent.mcp_tools_list_snapshot.v1",
        "benchmark": {
            "total_per_point": 1000,
            "concurrency_values": [40, 80],
            "p95_slo_ms": 400.0,
            "p99_slo_ms": 800.0,
        },
        "defaults": {"regression_factor": 2.0, "min_regression_delta_ms": 40.0},
        "targets": {
            "http://127.0.0.1:3002": {
                "recommended_concurrency": 80,
                "points": {
                    "80": {"baseline_p95_ms": 120.0, "baseline_p99_ms": 220.0},
                },
            }
        },
    }

    save_mcp_tools_list_snapshot(path, payload)
    loaded = load_mcp_tools_list_snapshot(path)
    assert loaded is not None
    assert loaded["schema"] == payload["schema"]
    assert loaded["targets"]["http://127.0.0.1:3002"]["points"]["80"][
        "baseline_p95_ms"
    ] == pytest.approx(120.0)
