#!/usr/bin/env python3
"""Report rendering for MCP startup stress probe."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def render_markdown(report: dict[str, object]) -> str:
    """Render human-readable markdown stress report."""
    summary = report["summary"]
    results = report["results"]
    lines = [
        "# MCP Startup Stress Report",
        "",
        "## Overview",
        f"- Started: `{report['started_at']}`",
        f"- Finished: `{report['finished_at']}`",
        f"- Duration: `{report['duration_ms']} ms`",
        f"- Total probes: `{summary['total']}`",
        f"- Passed: `{summary['passed']}`",
        f"- Failed: `{summary['failed']}`",
        f"- Pass rate: `{summary['pass_rate']:.2%}`",
        f"- Success avg startup: `{summary['success_avg_startup_ms']:.1f} ms`",
        f"- Success p95 startup: `{summary['success_p95_startup_ms']:.1f} ms`",
        "",
        "## Health Monitor",
        f"- Samples: `{summary['health_samples_total']}`",
        f"- Failed probes: `{summary['health_samples_failed']}`",
        f"- Failure rate: `{summary['health_failure_rate']:.2%}`",
        f"- Avg latency: `{summary['health_avg_latency_ms']:.1f} ms`",
        f"- P95 latency: `{summary['health_p95_latency_ms']:.1f} ms`",
        f"- Max latency: `{summary['health_max_latency_ms']:.1f} ms`",
        "",
        "## Failure Reasons",
    ]
    health_errors = summary.get("health_error_top", [])
    if isinstance(health_errors, list):
        lines.append("### Health Error Top")
        if health_errors:
            for item in health_errors:
                if not isinstance(item, dict):
                    continue
                detail = str(item.get("detail", "")).strip()
                count = int(item.get("count", 0))
                if detail:
                    lines.append(f"- `{count}` x `{detail}`")
        else:
            lines.append("- None")
        lines.append("")

    reason_counts = summary["reason_counts"]
    if reason_counts:
        for reason, count in sorted(reason_counts.items(), key=lambda item: item[0]):
            lines.append(f"- `{reason}`: `{count}`")
    else:
        lines.append("- None")

    lines.extend(
        [
            "",
            "## Probe Rows",
            (
                "| Round | Worker | Result | Reason | Startup ms | "
                "mcp.connect.succeeded | mcp.connect.failed |"
            ),
            "|---:|---:|---|---|---:|---:|---:|",
        ]
    )
    for row in results:
        lines.append(
            "| {round} | {worker} | {result} | {reason} | {startup} | {ok} | {failed} |".format(
                round=row["round_index"],
                worker=row["worker_index"],
                result="PASS" if row["success"] else "FAIL",
                reason=row["reason"],
                startup=row["startup_duration_ms"],
                ok=row["mcp_connect_succeeded"],
                failed=row["mcp_connect_failed"],
            )
        )

    failures = [row for row in results if not row["success"]]
    lines.extend(["", "## Failure Tails"])
    if not failures:
        lines.append("- None")
    else:
        for row in failures:
            lines.extend(
                [
                    (
                        f"### round={row['round_index']} worker={row['worker_index']} "
                        f"reason={row['reason']}"
                    ),
                    "```text",
                    row["tail"] or "(no logs)",
                    "```",
                ]
            )
    lines.append("")
    return "\n".join(lines)


def write_report(report: dict[str, object], output_json: Path, output_markdown: Path) -> None:
    """Write JSON and markdown reports to disk."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
