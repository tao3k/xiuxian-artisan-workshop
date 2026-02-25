#!/usr/bin/env python3
"""Unit tests for MCP tools/list concurrency sweep runtime helpers."""

from __future__ import annotations

import asyncio
import importlib
import sys
from pathlib import Path

import pytest

from omni.agent.mcp_server.tools_list_sweep import SweepPoint

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

runtime = importlib.import_module("mcp_tools_list_concurrency_sweep_runtime")


def test_parse_concurrency_values_sorts_and_deduplicates() -> None:
    assert runtime.parse_concurrency_values("80,40, 80,120") == [40, 80, 120]


def test_parse_concurrency_values_rejects_invalid_values() -> None:
    with pytest.raises(ValueError, match="positive integers"):
        runtime.parse_concurrency_values("1,0,2")
    with pytest.raises(ValueError, match="at least one concurrency value"):
        runtime.parse_concurrency_values(" , ")


def test_default_report_paths_use_host_and_port() -> None:
    json_out, markdown_out = runtime.default_report_paths("https://example.com")
    assert str(json_out).endswith(
        "mcp-tools-list-observability-example_com-443-concurrency-sweep.json"
    )
    assert str(markdown_out).endswith(
        "mcp-tools-list-observability-example_com-443-concurrency-sweep.md"
    )


def test_nearest_rank_percentile_handles_bounds() -> None:
    assert runtime.nearest_rank_percentile([], 0.95) == 0.0
    values = [1.0, 2.0, 3.0, 4.0]
    assert runtime.nearest_rank_percentile(values, 0.50) == 2.0
    assert runtime.nearest_rank_percentile(values, 0.99) == 3.0
    assert runtime.nearest_rank_percentile(values, 1.0) == 4.0


def test_build_markdown_includes_recommendation() -> None:
    markdown = runtime.build_markdown(
        base_url="http://127.0.0.1:3002",
        points=[
            SweepPoint(
                concurrency=40,
                total=1000,
                errors=0,
                elapsed_s=2.0,
                rps=500.0,
                p50_ms=12.0,
                p95_ms=24.0,
                p99_ms=30.0,
            )
        ],
        p95_slo_ms=400.0,
        p99_slo_ms=800.0,
        recommendation_concurrency=40,
        recommendation_reason="within target",
        knee_concurrency=80,
    )
    assert "MCP tools/list Concurrency Sweep" in markdown
    assert "Recommended concurrency: `40`" in markdown
    assert "within target" in markdown


def test_run_benchmark_collects_errors_and_percentiles() -> None:
    async def _fake_call(_client, _rpc_url, request_id: int) -> float:
        if request_id % 2 == 0:
            raise RuntimeError("boom")
        return 10.0 + float(request_id % 3)

    result = asyncio.run(
        runtime.run_benchmark(
            client=object(),
            rpc_url="http://127.0.0.1:3002/",
            total=6,
            concurrency=3,
            start_id=1,
            sweep_point_cls=SweepPoint,
            call_tools_list_fn=_fake_call,
        )
    )
    assert result.total == 6
    assert result.concurrency == 3
    assert result.errors == 3
    assert result.p50_ms > 0.0
    assert result.p95_ms >= result.p50_ms
