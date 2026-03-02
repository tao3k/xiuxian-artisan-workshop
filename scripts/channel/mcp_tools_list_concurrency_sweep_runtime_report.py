#!/usr/bin/env python3
"""Report rendering/path helpers for MCP tools/list concurrency sweep."""

from __future__ import annotations

from pathlib import Path
from typing import Any
from urllib.parse import urlparse


def default_report_paths(base_url: str) -> tuple[Path, Path]:
    """Build default JSON/Markdown report paths for a target base URL."""
    parsed = urlparse(base_url)
    host = parsed.hostname or "unknown-host"
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    stem = f"mcp-tools-list-observability-{host.replace('.', '_')}-{port}-concurrency-sweep"
    report_root = Path(".run/reports")
    return report_root / f"{stem}.json", report_root / f"{stem}.md"


def build_markdown(
    *,
    base_url: str,
    points: list[Any],
    p95_slo_ms: float,
    p99_slo_ms: float,
    recommendation_concurrency: int | None,
    recommendation_reason: str,
    knee_concurrency: int | None,
) -> str:
    """Render markdown summary for a concurrency sweep report."""
    lines = [
        f"# MCP tools/list Concurrency Sweep ({base_url})",
        "",
        f"SLO target: `p95 <= {p95_slo_ms}ms`, `p99 <= {p99_slo_ms}ms`",
        "",
        "| Concurrency | Total | Errors | RPS | p50 (ms) | p95 (ms) | p99 (ms) |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for point in points:
        lines.append(
            f"| {point.concurrency} | {point.total} | {point.errors} | {point.rps} | "
            f"{point.p50_ms} | {point.p95_ms} | {point.p99_ms} |"
        )
    lines.extend(
        [
            "",
            f"Estimated knee concurrency: `{knee_concurrency}`"
            if knee_concurrency
            else "Estimated knee concurrency: `not detected`",
            f"Recommended concurrency: `{recommendation_concurrency}`"
            if recommendation_concurrency
            else "Recommended concurrency: `none`",
            f"Recommendation reason: {recommendation_reason}",
        ]
    )
    return "\n".join(lines) + "\n"
