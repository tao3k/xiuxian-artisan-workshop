#!/usr/bin/env python3
"""Report rendering helpers for MCP startup suite."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def render_markdown(report: dict[str, object]) -> str:
    """Render suite report markdown."""
    quality = report.get("quality_gate")
    if not isinstance(quality, dict):
        quality = {"passed": True, "violations": [], "thresholds": {}}
    quality_thresholds = quality.get("thresholds") if isinstance(quality, dict) else {}
    if not isinstance(quality_thresholds, dict):
        quality_thresholds = {}
    quality_violations = quality.get("violations") if isinstance(quality, dict) else []
    if not isinstance(quality_violations, list):
        quality_violations = []

    lines = [
        "# MCP Startup Suite Report",
        "",
        "## Overview",
        f"- Started: `{report['started_at']}`",
        f"- Finished: `{report['finished_at']}`",
        f"- Duration: `{report['duration_ms']} ms`",
        f"- Overall passed: `{report['overall_passed']}`",
        f"- Passed modes: `{report['passed_modes']}`",
        f"- Failed modes: `{report['failed_modes']}`",
        f"- Quality gate passed: `{quality.get('passed', True)}`",
        "",
        "## Quality Gate",
        f"- max_failed_probes: `{quality_thresholds.get('max_failed_probes', 0)}`",
        f"- max_hot_p95_ms: `{float(quality_thresholds.get('max_hot_p95_ms', 0.0)):.1f}`",
        f"- max_cold_p95_ms: `{float(quality_thresholds.get('max_cold_p95_ms', 0.0)):.1f}`",
        f"- min_health_samples: `{int(quality_thresholds.get('min_health_samples', 0))}`",
        (
            "- max_health_failure_rate: "
            f"`{float(quality_thresholds.get('max_health_failure_rate', 0.0)):.2%}`"
        ),
        f"- max_health_p95_ms: `{float(quality_thresholds.get('max_health_p95_ms', 0.0)):.1f}`",
        (
            "- max_hot_p95_regression_ratio: "
            f"`{float(quality_thresholds.get('max_hot_p95_regression_ratio', 0.0)):.2%}`"
        ),
        (
            "- max_cold_p95_regression_ratio: "
            f"`{float(quality_thresholds.get('max_cold_p95_regression_ratio', 0.0)):.2%}`"
        ),
        "",
        "## Modes",
        (
            "| Mode | Result | Rounds | Parallel | Avg ms | P95 ms | Failed Probes | "
            "Health Fail Rate | Health P95 ms | Report |"
        ),
        "|---|---|---:|---:|---:|---:|---:|---:|---:|---|",
    ]
    for mode in report["modes"]:
        summary = mode.get("summary") or {}
        lines.append(
            (
                "| {mode_name} | {result} | {rounds} | {parallel} | {avg:.1f} | {p95:.1f} | "
                "{failed} | {health_fail_rate:.2%} | {health_p95:.1f} | `{report_json}` |"
            ).format(
                mode_name=mode["mode"],
                result="PASS" if mode["passed"] else "FAIL",
                rounds=mode["rounds"],
                parallel=mode["parallel"],
                avg=float(summary.get("success_avg_startup_ms", 0.0)),
                p95=float(summary.get("success_p95_startup_ms", 0.0)),
                failed=int(summary.get("failed", 0)),
                health_fail_rate=float(summary.get("health_failure_rate", 0.0)),
                health_p95=float(summary.get("health_p95_latency_ms", 0.0)),
                report_json=mode["json_report"],
            )
        )

    lines.extend(["", "## Quality Violations"])
    if not quality_violations:
        lines.append("- None")
    else:
        for violation in quality_violations:
            lines.append(f"- {violation}")

    failures = [mode for mode in report["modes"] if not mode["passed"]]
    lines.extend(["", "## Failure Tails"])
    if not failures:
        lines.append("- None")
    else:
        for mode in failures:
            lines.extend(
                [
                    f"### {mode['mode']}",
                    "```text",
                    mode.get("stdout_tail", "") or "(no stdout)",
                    mode.get("stderr_tail", "") or "(no stderr)",
                    "```",
                ]
            )
    lines.append("")
    return "\n".join(lines)


def write_report(report: dict[str, object], output_json: Path, output_markdown: Path) -> None:
    """Write JSON and markdown suite reports."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
