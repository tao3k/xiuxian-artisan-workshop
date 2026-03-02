#!/usr/bin/env python3
"""Markdown rendering helpers for command event probe reports."""

from __future__ import annotations


def _build_attempt_rows(report: dict[str, object]) -> list[str]:
    """Render attempt table rows for markdown output."""
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
    return rows


def render_markdown(report: dict[str, object]) -> str:
    """Render a markdown report from structured command-events payload."""
    summary = report["summary"]
    config = report["config"]
    rows = _build_attempt_rows(report)
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
