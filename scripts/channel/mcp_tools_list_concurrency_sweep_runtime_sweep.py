#!/usr/bin/env python3
"""Top-level sweep execution helper for MCP tools/list concurrency probe."""

from __future__ import annotations

import statistics
from dataclasses import asdict
from typing import Any

import httpx
from mcp_tools_list_concurrency_sweep_runtime_benchmark import call_tools_list, run_benchmark
from mcp_tools_list_concurrency_sweep_runtime_stats import (
    normalize_base_url,
    parse_concurrency_values,
)


async def run_sweep(
    args: Any,
    *,
    sweep_point_cls: Any,
    recommended_http_pool_limits_fn: Any,
    recommend_concurrency_by_slo_fn: Any,
    call_tools_list_fn: Any = call_tools_list,
    run_benchmark_fn: Any = run_benchmark,
) -> dict[str, object]:
    """Execute full concurrency sweep and return JSON-serializable summary."""
    base_url = normalize_base_url(args.base_url)
    health_url = f"{base_url}/health"
    rpc_url = f"{base_url}/"
    timeout = httpx.Timeout(args.timeout_secs)

    concurrency_values = parse_concurrency_values(args.concurrency_values)
    max_connections, max_keepalive_connections = recommended_http_pool_limits_fn(
        max(concurrency_values)
    )
    if args.total <= 0:
        raise ValueError("--total must be positive")
    if args.warmup_calls < 0:
        raise ValueError("--warmup-calls must be >= 0")

    limits = httpx.Limits(
        max_connections=max_connections,
        max_keepalive_connections=max_keepalive_connections,
    )
    async with httpx.AsyncClient(timeout=timeout, limits=limits) as client:
        health_response = await client.get(health_url)
        health_response.raise_for_status()
        health_payload = health_response.json()

        for warmup_idx in range(args.warmup_calls):
            await call_tools_list_fn(client, rpc_url, warmup_idx + 1)

        points = []
        for index, concurrency in enumerate(concurrency_values):
            point = await run_benchmark_fn(
                client,
                rpc_url,
                total=args.total,
                concurrency=concurrency,
                start_id=10_000 + (index * 100_000),
                sweep_point_cls=sweep_point_cls,
                call_tools_list_fn=call_tools_list_fn,
            )
            points.append(point)

    recommendation = recommend_concurrency_by_slo_fn(
        points,
        p95_slo_ms=args.p95_slo_ms,
        p99_slo_ms=args.p99_slo_ms,
    )

    mean_rps = round(statistics.mean(point.rps for point in points), 2)
    error_total = sum(point.errors for point in points)
    return {
        "base_url": base_url,
        "health_ok": True,
        "health_status": health_payload.get("status"),
        "slo": {"p95_ms": args.p95_slo_ms, "p99_ms": args.p99_slo_ms},
        "total_per_point": args.total,
        "concurrency_values": concurrency_values,
        "http_client_limits": {
            "max_connections": max_connections,
            "max_keepalive_connections": max_keepalive_connections,
        },
        "points": [asdict(point) for point in points],
        "summary": {
            "point_count": len(points),
            "error_total": error_total,
            "mean_rps": mean_rps,
        },
        "recommendation": {
            "recommended_concurrency": recommendation.recommended_concurrency,
            "reason": recommendation.reason,
            "feasible_concurrency": list(recommendation.feasible_concurrency),
            "knee_concurrency": recommendation.knee_concurrency,
        },
    }
