#!/usr/bin/env python3
"""Top-level markdown renderer for complex scenario reports."""

from __future__ import annotations

from complex_scenarios_report_sections import (
    append_failure_tails,
    append_mcp_diagnostics,
    append_memory_adaptation,
    append_natural_language_trace,
    append_scenario_header,
    append_step_table,
)


def render_markdown(report: dict[str, object]) -> str:
    """Render markdown report for complex scenario execution results."""
    lines: list[str] = [
        "# Agent Channel Complex Scenario Report",
        "",
        "## Overview",
        f"- started_at: `{report['started_at']}`",
        f"- finished_at: `{report['finished_at']}`",
        f"- duration_ms: `{report['duration_ms']}`",
        f"- overall: `{'PASS' if report['overall_passed'] else 'FAIL'}`",
        f"- scenarios: `{report['summary']['passed']}/{report['summary']['total']}` passed",
        f"- runtime_partition_mode: `{report['config']['runtime_partition_mode']}`",
        "",
        "## Sessions",
    ]

    for session in report["config"]["sessions"]:
        lines.append(
            "- `{alias}` -> chat_id=`{chat}` user_id=`{user}` thread_id=`{thread}` "
            "chat_title=`{title}`".format(
                alias=session["alias"],
                chat=session["chat_id"],
                user=session["user_id"],
                thread=session["thread_id"],
                title=session["chat_title"],
            )
        )

    lines.extend(["", "## Scenario Results", ""])
    for scenario in report["scenarios"]:
        append_scenario_header(lines, scenario)
        append_step_table(lines, scenario)
        append_natural_language_trace(lines, scenario)
        append_memory_adaptation(lines, scenario)
        append_mcp_diagnostics(lines, scenario)
        append_failure_tails(lines, scenario)
        lines.append("")

    return "\n".join(lines)
