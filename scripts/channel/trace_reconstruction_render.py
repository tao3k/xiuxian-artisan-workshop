#!/usr/bin/env python3
"""Markdown rendering for trace reconstruction output."""

from __future__ import annotations

from typing import Any


def render_markdown_report(
    entries: list[dict[str, Any]],
    summary: dict[str, Any],
    *,
    title: str = "Omni Agent Trace Reconstruction",
) -> str:
    """Render a human-readable markdown report from trace payload."""
    lines: list[str] = [f"# {title}", ""]
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- Events: {summary['events_total']}")
    lines.append(f"- Quality score: {summary['quality_score']}")
    lines.append("")
    lines.append("### Stage Flags")
    for key, value in summary["stage_flags"].items():
        lines.append(f"- {key}: {value}")

    warnings = summary.get("warnings", [])
    lines.append("")
    lines.append("### Warnings")
    if warnings:
        for warning in warnings:
            lines.append(f"- {warning}")
    else:
        lines.append("- none")

    lines.append("")
    lines.append("## Timeline")
    lines.append("")
    lines.append("| Line | Time | Level | Event | Notes |")
    lines.append("| --- | --- | --- | --- | --- |")
    for entry in entries:
        notes = []
        for key in (
            "session_id",
            "session_key",
            "chat_id",
            "route",
            "confidence",
            "verdict",
            "injection_mode",
            "role_mix_profile_id",
        ):
            value = entry["fields"].get(key)
            if value:
                notes.append(f"{key}={value}")
        lines.append(
            f"| {entry['line']} | {entry.get('timestamp') or ''} | {entry.get('level') or ''} "
            f"| `{entry['event']}` | {'; '.join(notes)} |"
        )
    return "\n".join(lines) + "\n"
