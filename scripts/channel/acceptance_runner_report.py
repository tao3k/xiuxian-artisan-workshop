#!/usr/bin/env python3
"""Report rendering helpers for acceptance runner."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def to_markdown(report: dict[str, object]) -> str:
    """Render acceptance report as markdown."""
    lines: list[str] = [
        "# Agent Channel Acceptance Report",
        "",
        "## Overview",
        f"- started_at: `{report['started_at']}`",
        f"- finished_at: `{report['finished_at']}`",
        f"- duration_ms: `{report['duration_ms']}`",
        f"- overall: `{'PASS' if report['overall_passed'] else 'FAIL'}`",
        (
            f"- steps: `{report['summary']['passed']}/{report['summary']['total']}` passed"  # type: ignore[index]
        ),
        "",
        "## Outputs",
        f"- group_profile_json: `{report['artifacts']['group_profile_json']}`",  # type: ignore[index]
        f"- group_profile_env: `{report['artifacts']['group_profile_env']}`",  # type: ignore[index]
        f"- matrix_json: `{report['artifacts']['matrix_json']}`",  # type: ignore[index]
        f"- complex_json: `{report['artifacts']['complex_json']}`",  # type: ignore[index]
        f"- memory_evolution_json: `{report['artifacts']['memory_evolution_json']}`",  # type: ignore[index]
        "",
        "## Step Results",
        "| Step | Result | Return Code | Attempts | Duration (ms) |",
        "|---|---|---:|---:|---:|",
    ]

    for step in report["steps"]:  # type: ignore[index]
        status = "PASS" if step["passed"] else "FAIL"
        lines.append(
            f"| `{step['step']}` | {status} | {step['returncode']} | "
            f"{step['attempts']} | {step['duration_ms']} |"
        )

    failed_steps = [step for step in report["steps"] if not step["passed"]]  # type: ignore[index]
    lines.append("")
    lines.append("## Failure Tails")
    if not failed_steps:
        lines.append("- None")
        return "\n".join(lines) + "\n"

    for step in failed_steps:
        lines.append(f"### {step['step']}")
        lines.append("")
        lines.append("```text")
        if step["missing_outputs"]:
            lines.append(f"missing_outputs={step['missing_outputs']}")
        if step["stderr_tail"]:
            lines.append(step["stderr_tail"])
        elif step["stdout_tail"]:
            lines.append(step["stdout_tail"])
        lines.append("```")
        lines.append("")

    return "\n".join(lines)


def write_report(report: dict[str, object], *, output_json: Path, output_markdown: Path) -> None:
    """Write report JSON + markdown files."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(
        json.dumps(report, ensure_ascii=False, indent=2, default=str),
        encoding="utf-8",
    )
    output_markdown.write_text(to_markdown(report), encoding="utf-8")
