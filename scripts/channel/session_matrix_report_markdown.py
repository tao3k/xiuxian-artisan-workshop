#!/usr/bin/env python3
"""Markdown rendering helpers for session-matrix reports."""

from __future__ import annotations


def _build_step_rows(report: dict[str, object]) -> list[str]:
    """Render markdown table rows for each matrix step."""
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
    return rows


def _build_failure_blocks(report: dict[str, object]) -> list[str]:
    """Render markdown failure-tail blocks for failed steps."""
    blocks = [
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
    if not blocks:
        return ["- None"]
    return blocks


def render_markdown(report: dict[str, object]) -> str:
    """Render markdown report for session matrix runs."""
    summary = report["summary"]
    config = report["config"]
    rows = _build_step_rows(report)
    failure_blocks = _build_failure_blocks(report)

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
