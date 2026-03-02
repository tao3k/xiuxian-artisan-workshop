#!/usr/bin/env python3
"""Confidence-section builder for memory benchmark markdown reports."""

from __future__ import annotations

from typing import Any


def build_confidence_lines(*, mode_summaries: dict[str, Any]) -> list[str]:
    """Build confidence section lines."""
    confidence_note = "moderate"
    baseline = mode_summaries.get("baseline")
    adaptive = mode_summaries.get("adaptive")
    if baseline and adaptive and min(baseline.scored_turns, adaptive.scored_turns) < 5:
        confidence_note = "low"
    total_mcp_error_turns = sum(summary.mcp_error_turns for summary in mode_summaries.values())
    total_embedding_fallback_turns = sum(
        summary.embedding_fallback_turns_total for summary in mode_summaries.values()
    )

    lines = [
        "## Confidence",
        "",
        "- This benchmark is proxy-based (keyword hit + memory observability metrics), "
        "not a human-graded semantic evaluation.",
        f"- Confidence level for this run: `{confidence_note}` "
        f"(scored turns may be small; increase `--iterations` for stronger signal).",
    ]
    if total_mcp_error_turns > 0:
        lines.append(
            f"- MCP error interference observed on `{total_mcp_error_turns}` query turn(s)."
        )
    if total_embedding_fallback_turns > 0:
        lines.append(
            f"- Embedding fallback observed on `{total_embedding_fallback_turns}` query turn(s)."
        )
    lines.append("")
    return lines
