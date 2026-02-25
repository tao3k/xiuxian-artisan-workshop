#!/usr/bin/env python3
"""Report rendering for Discord ingress stress probe."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def render_markdown(report: dict[str, object]) -> str:
    """Render Markdown report for stress results."""
    summary = report["summary"]
    inputs = report["inputs"]
    rounds = report["rounds"]

    lines = [
        "# Discord Ingress Stress Report",
        "",
        "## Overview",
        f"- Started: `{report['started_at']}`",
        f"- Finished: `{report['finished_at']}`",
        f"- Duration: `{report['duration_ms']} ms`",
        f"- Ingress URL: `{inputs['ingress_url']}`",
        f"- Measured rounds: `{summary['measured_rounds']}`",
        f"- Total requests: `{summary['total_requests']}`",
        f"- Success requests: `{summary['success_requests']}`",
        f"- Failed requests: `{summary['failed_requests']}`",
        f"- Failure rate: `{summary['failure_rate']:.2%}`",
        f"- Average RPS: `{summary['average_rps']:.2f}`",
        f"- Max round p95 latency: `{summary['max_round_p95_ms']:.2f} ms`",
        "",
        "## Queue Pressure Signals",
        f"- Parsed ingress messages: `{summary['parsed_messages']}`",
        f"- Ingress queue wait events: `{summary['queue_wait_events']}`",
        f"- Foreground gate wait events: `{summary['foreground_gate_wait_events']}`",
        (f"- Inbound queue unavailable events: `{summary['inbound_queue_unavailable_events']}`"),
        "",
        "## Quality Gate",
        f"- Passed: `{summary['quality_passed']}`",
    ]

    failures = summary.get("quality_failures", [])
    if isinstance(failures, list) and failures:
        lines.append("- Failures:")
        for item in failures:
            lines.append(f"  - {item}")
    else:
        lines.append("- Failures: `None`")

    lines.extend(
        [
            "",
            "## Round Rows",
            (
                "| Round | Warmup | Requests | Success | Failed | p95 ms | "
                "RPS | queue_wait | gate_wait | queue_unavailable |"
            ),
            "|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|",
        ]
    )

    for row in rounds:
        lines.append(
            "| {round} | {warmup} | {req} | {ok} | {failed} | {p95:.2f} | {rps:.2f} | "
            "{qwait} | {gwait} | {qun} |".format(
                round=row["round_index"],
                warmup="yes" if row["warmup"] else "no",
                req=row["total_requests"],
                ok=row["success_requests"],
                failed=row["failed_requests"],
                p95=row["p95_latency_ms"],
                rps=row["rps"],
                qwait=row["log_queue_wait_events"],
                gwait=row["log_foreground_gate_wait_events"],
                qun=row["log_inbound_queue_unavailable_events"],
            )
        )

    lines.append("")
    return "\n".join(lines)


def write_report(report: dict[str, object], output_json: Path, output_markdown: Path) -> None:
    """Write JSON + Markdown reports to disk."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
