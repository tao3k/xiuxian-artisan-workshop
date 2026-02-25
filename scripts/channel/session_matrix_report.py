#!/usr/bin/env python3
"""Report helpers for session-matrix black-box probes."""

from __future__ import annotations

import json
import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


def render_markdown(report: dict[str, object]) -> str:
    """Render markdown report for session matrix runs."""
    summary = report["summary"]
    config = report["config"]
    rows: list[str] = [
        "| Step | Kind | Session | Prompt | Event | Result | Duration (ms) |",
        "|---|---|---|---|---|---|---:|",
    ]
    for step in report["steps"]:
        status = "PASS" if step["passed"] else "FAIL"
        rows.append(
            "| {name} | {kind} | `{session}` | `{prompt}` | `{event}` | {status} | {duration} |".format(
                name=step["name"],
                kind=step["kind"],
                session=step["session_key"] or "-",
                prompt=step["prompt"] or "-",
                event=step["event"] or "-",
                status=status,
                duration=step["duration_ms"],
            )
        )
    failure_blocks = [
        "\n".join(
            [
                f"### {step['name']}",
                "",
                "```text",
                (step["stderr_tail"] or step["stdout_tail"] or "(no output)"),
                "```",
            ]
        )
        for step in report["steps"]
        if not step["passed"]
    ]
    if not failure_blocks:
        failure_blocks = ["- None"]

    return "\n".join(
        [
            "# Agent Channel Session Matrix Report",
            "",
            "## Overview",
            f"- Started: `{report['started_at']}`",
            f"- Finished: `{report['finished_at']}`",
            f"- Duration: `{report['duration_ms']} ms`",
            f"- Overall: `{'PASS' if report['overall_passed'] else 'FAIL'}`",
            f"- Steps: `{summary['passed']}/{summary['total']}` passed",
            "",
            "## Session Inputs",
            f"- chat_id: `{config['chat_id']}`",
            f"- chat_b: `{config['chat_b']}`",
            f"- chat_c: `{config['chat_c']}`",
            f"- user_a: `{config['user_a']}`",
            f"- user_b: `{config['user_b']}`",
            f"- user_c: `{config['user_c']}`",
            f"- thread_a: `{config['thread_a']}`",
            f"- thread_b: `{config['thread_b']}`",
            f"- thread_c: `{config['thread_c']}`",
            f"- log_file: `{config['log_file']}`",
            "",
            "## Step Results",
            *rows,
            "",
            "## Failure Tails",
            *failure_blocks,
            "",
        ]
    )


def build_report(
    cfg: Any,
    results: list[Any],
    started_dt: datetime,
    started_mono: float,
) -> dict[str, object]:
    """Build structured report payload."""
    finished_dt = datetime.now(UTC)
    duration_ms = int((time.monotonic() - started_mono) * 1000)
    passed = sum(1 for result in results if result.passed)
    failed = len(results) - passed
    report: dict[str, object] = {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": duration_ms,
        "overall_passed": failed == 0 and len(results) > 0,
        "summary": {"total": len(results), "passed": passed, "failed": failed},
        "config": {
            "webhook_url": cfg.webhook_url,
            "log_file": str(cfg.log_file),
            "chat_id": cfg.chat_id,
            "chat_b": cfg.chat_b,
            "chat_c": cfg.chat_c,
            "user_a": cfg.user_a,
            "user_b": cfg.user_b,
            "user_c": cfg.user_c,
            "thread_a": cfg.thread_a,
            "thread_b": cfg.thread_b,
            "thread_c": cfg.thread_c,
            "mixed_plain_prompt": cfg.mixed_plain_prompt,
            "forbid_log_regexes": list(cfg.forbid_log_regexes),
        },
        "steps": [asdict(result) for result in results],
    }
    return report


def write_outputs(report: dict[str, object], output_json: Any, output_markdown: Any) -> None:
    """Write JSON + Markdown artifacts."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
