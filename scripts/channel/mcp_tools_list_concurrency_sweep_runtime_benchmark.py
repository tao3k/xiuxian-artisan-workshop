#!/usr/bin/env python3
"""Benchmark execution helpers for MCP tools/list concurrency sweep."""

from __future__ import annotations

import asyncio
import time
from typing import TYPE_CHECKING, Any

from mcp_tools_list_concurrency_sweep_runtime_stats import nearest_rank_percentile

if TYPE_CHECKING:
    import httpx


async def call_tools_list(
    client: httpx.AsyncClient,
    rpc_url: str,
    request_id: int,
) -> float:
    """Call tools/list RPC and return request latency in milliseconds."""
    started = time.perf_counter()
    response = await client.post(
        rpc_url,
        json={"jsonrpc": "2.0", "id": request_id, "method": "tools/list", "params": {}},
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    response.raise_for_status()
    payload = response.json()
    if payload.get("error") is not None:
        raise RuntimeError(f"tools/list returned error: {payload['error']}")
    result = payload.get("result")
    if not isinstance(result, dict):
        raise RuntimeError("tools/list result is not an object")
    tools = result.get("tools")
    if not isinstance(tools, list):
        raise RuntimeError("tools/list result.tools is not a list")
    return elapsed_ms


async def run_benchmark(
    client: httpx.AsyncClient,
    rpc_url: str,
    *,
    total: int,
    concurrency: int,
    start_id: int,
    sweep_point_cls: Any,
    call_tools_list_fn: Any = call_tools_list,
) -> Any:
    """Run one concurrent tools/list benchmark point."""
    semaphore = asyncio.Semaphore(concurrency)
    latencies_ms: list[float] = []
    errors = 0

    async def one(index: int) -> None:
        nonlocal errors
        async with semaphore:
            try:
                elapsed_ms = await call_tools_list_fn(client, rpc_url, start_id + index)
            except Exception:
                errors += 1
                return
            latencies_ms.append(elapsed_ms)

    started = time.perf_counter()
    await asyncio.gather(*(one(i) for i in range(total)))
    elapsed_s = time.perf_counter() - started

    sorted_lat = sorted(latencies_ms)
    p50_ms = nearest_rank_percentile(sorted_lat, 0.50)
    p95_ms = nearest_rank_percentile(sorted_lat, 0.95)
    p99_ms = nearest_rank_percentile(sorted_lat, 0.99)
    rps = total / elapsed_s if elapsed_s > 0 else 0.0

    return sweep_point_cls(
        concurrency=concurrency,
        total=total,
        errors=errors,
        elapsed_s=round(elapsed_s, 3),
        rps=round(rps, 2),
        p50_ms=round(p50_ms, 2),
        p95_ms=round(p95_ms, 2),
        p99_ms=round(p99_ms, 2),
    )
