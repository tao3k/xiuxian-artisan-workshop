#!/usr/bin/env python3
"""Report helpers for agent channel command event probes."""

from __future__ import annotations

import json
import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


def render_markdown(report: dict[str, object]) -> str:
    """Render a markdown report from structured command-events payload."""
    summary = report["summary"]
    config = report["config"]
    rows: list[str] = [
        "| Mode | Case | Chat | Attempt | Result | Return | Duration (ms) | Retry Scheduled |",
        "|---|---|---:|---:|---|---:|---:|---|",
    ]
    for attempt in report["attempts"]:
        rows.append(
            "| {mode} | `{case_id}` | {chat} | {attempt}/{max_attempts} | {result} | {rc} | {dur} | {retry} |".format(
                mode=attempt["mode"],
                case_id=attempt["case_id"],
                chat=attempt["chat_id"] if attempt["chat_id"] is not None else "-",
                attempt=attempt["attempt"],
                max_attempts=attempt["max_attempts"],
                result="PASS" if attempt["passed"] else "FAIL",
                rc=attempt["returncode"],
                dur=attempt["duration_ms"],
                retry="yes" if attempt["retry_scheduled"] else "no",
            )
        )
    return "\n".join(
        [
            "# Agent Channel Command Events Report",
            "",
            "## Overview",
            f"- Started: `{report['started_at']}`",
            f"- Finished: `{report['finished_at']}`",
            f"- Duration: `{report['duration_ms']} ms`",
            f"- Overall: `{'PASS' if report['overall_passed'] else 'FAIL'}`",
            f"- Attempts: `{summary['passed']}/{summary['total']}` passed",
            f"- Retried attempts: `{summary['retry_scheduled']}`",
            "",
            "## Config",
            f"- suite: `{config['suites']}`",
            f"- runtime_partition_mode: `{config['runtime_partition_mode']}`",
            f"- admin_matrix: `{config['admin_matrix']}`",
            f"- assert_admin_isolation: `{config['assert_admin_isolation']}`",
            f"- assert_admin_topic_isolation: `{config['assert_admin_topic_isolation']}`",
            f"- group_thread_id: `{config['group_thread_id']}`",
            f"- group_thread_id_b: `{config['group_thread_id_b']}`",
            f"- matrix_chat_ids: `{config['matrix_chat_ids']}`",
            f"- max_wait: `{config['max_wait']}`",
            f"- max_idle_secs: `{config['max_idle_secs']}`",
            "",
            "## Attempts",
            *rows,
            "",
        ]
    )


def build_report(
    *,
    suites: tuple[str, ...],
    case_ids: tuple[str, ...],
    allow_chat_ids: tuple[str, ...],
    matrix_chat_ids: tuple[int, ...],
    attempts: list[Any],
    started_dt: datetime,
    started_mono: float,
    exit_code: int,
    runtime_partition_mode: str | None,
    admin_matrix: bool,
    assert_admin_isolation: bool,
    assert_admin_topic_isolation: bool,
    group_thread_id: int | None,
    group_thread_id_b: int | None,
    max_wait: int,
    max_idle_secs: int,
    matrix_retries: int,
    matrix_backoff_secs: float,
) -> dict[str, object]:
    """Build structured report payload from probe attempts and run config."""
    finished_dt = datetime.now(UTC)
    duration_ms = int((time.monotonic() - started_mono) * 1000)
    passed = sum(1 for attempt in attempts if attempt.passed)
    failed = len(attempts) - passed
    retry_scheduled = sum(1 for attempt in attempts if attempt.retry_scheduled)
    return {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": duration_ms,
        "exit_code": exit_code,
        "overall_passed": exit_code == 0 and failed == 0 and len(attempts) > 0,
        "summary": {
            "total": len(attempts),
            "passed": passed,
            "failed": failed,
            "retry_scheduled": retry_scheduled,
        },
        "config": {
            "suites": list(suites),
            "cases": list(case_ids),
            "runtime_partition_mode": runtime_partition_mode,
            "admin_matrix": bool(admin_matrix),
            "assert_admin_isolation": bool(assert_admin_isolation),
            "assert_admin_topic_isolation": bool(assert_admin_topic_isolation),
            "group_thread_id": group_thread_id,
            "group_thread_id_b": group_thread_id_b,
            "matrix_chat_ids": list(matrix_chat_ids),
            "allow_chat_ids": list(allow_chat_ids),
            "max_wait": int(max_wait),
            "max_idle_secs": int(max_idle_secs),
            "matrix_retries": int(matrix_retries),
            "matrix_backoff_secs": float(matrix_backoff_secs),
        },
        "attempts": [asdict(attempt) for attempt in attempts],
    }


def write_outputs(report: dict[str, object], output_json: Any, output_markdown: Any) -> None:
    """Write JSON and Markdown reports to output paths."""
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_markdown.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    output_markdown.write_text(render_markdown(report), encoding="utf-8")
