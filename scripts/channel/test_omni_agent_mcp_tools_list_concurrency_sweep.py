#!/usr/bin/env python3
"""
Run MCP tools/list concurrency sweep and emit recommendation by SLO.

Outputs:
- JSON report: machine-readable sweep metrics + recommendation
- Markdown report: concise table for quick review
"""

from __future__ import annotations

import argparse
import asyncio
import importlib
import json
import sys
from pathlib import Path

from omni.agent.mcp_server.tools_list_sweep import (
    SweepPoint,
    recommend_concurrency_by_slo,
    recommended_http_pool_limits,
)

_runtime_module = importlib.import_module("mcp_tools_list_concurrency_sweep_runtime")

_normalize_base_url = _runtime_module.normalize_base_url
_default_report_paths = _runtime_module.default_report_paths
_build_markdown = _runtime_module.build_markdown


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run MCP tools/list concurrency sweep and output recommendation."
    )
    parser.add_argument("--base-url", default="http://127.0.0.1:3002")
    parser.add_argument("--timeout-secs", type=float, default=30.0)
    parser.add_argument("--concurrency-values", default="40,80,120,160,200")
    parser.add_argument("--total", type=int, default=1000, help="Requests per concurrency point.")
    parser.add_argument("--warmup-calls", type=int, default=2)
    parser.add_argument("--p95-slo-ms", type=float, default=400.0)
    parser.add_argument("--p99-slo-ms", type=float, default=800.0)
    parser.add_argument("--json-out", type=Path, default=None)
    parser.add_argument("--markdown-out", type=Path, default=None)
    parser.add_argument(
        "--allow-request-errors",
        action="store_true",
        help="Do not fail process exit when request errors are present.",
    )
    return parser.parse_args()


async def _run(args: argparse.Namespace) -> dict[str, object]:
    return await _runtime_module.run_sweep(
        args,
        sweep_point_cls=SweepPoint,
        recommended_http_pool_limits_fn=recommended_http_pool_limits,
        recommend_concurrency_by_slo_fn=recommend_concurrency_by_slo,
    )


def main() -> int:
    args = _parse_args()
    default_json_out, default_markdown_out = _default_report_paths(
        _normalize_base_url(args.base_url)
    )
    json_out = args.json_out or default_json_out
    markdown_out = args.markdown_out or default_markdown_out

    try:
        summary = asyncio.run(_run(args))
    except Exception as exc:
        print(f"sweep_failed: {exc}", file=sys.stderr)
        return 1

    points = [SweepPoint(**point) for point in summary["points"]]  # type: ignore[arg-type]
    recommendation = summary["recommendation"]  # type: ignore[assignment]

    print("=== MCP tools/list concurrency sweep ===")
    print(f"base_url: {summary['base_url']}")
    print(f"concurrency_values: {summary['concurrency_values']}")
    print(f"http_client_limits: {summary['http_client_limits']}")
    print(f"total_per_point: {summary['total_per_point']}")
    print(f"slo: {summary['slo']}")
    for point in points:
        print(
            "point: "
            f"c={point.concurrency} total={point.total} err={point.errors} "
            f"rps={point.rps} p50={point.p50_ms} p95={point.p95_ms} p99={point.p99_ms}"
        )
    print(
        "recommendation: "
        f"concurrency={recommendation['recommended_concurrency']} "
        f"knee={recommendation['knee_concurrency']} "
        f"reason={recommendation['reason']}"
    )

    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    markdown_out.parent.mkdir(parents=True, exist_ok=True)
    markdown_out.write_text(
        _build_markdown(
            base_url=str(summary["base_url"]),
            points=points,
            p95_slo_ms=float(summary["slo"]["p95_ms"]),  # type: ignore[index]
            p99_slo_ms=float(summary["slo"]["p99_ms"]),  # type: ignore[index]
            recommendation_concurrency=recommendation["recommended_concurrency"],  # type: ignore[index]
            recommendation_reason=str(recommendation["reason"]),  # type: ignore[index]
            knee_concurrency=recommendation["knee_concurrency"],  # type: ignore[index]
        ),
        encoding="utf-8",
    )
    print(f"json_out: {json_out}")
    print(f"markdown_out: {markdown_out}")
    print("--- summary_json ---")
    print(json.dumps(summary, ensure_ascii=False))

    if int(summary["summary"]["error_total"]) > 0 and not args.allow_request_errors:  # type: ignore[index]
        print("sweep_failed: request errors detected", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
