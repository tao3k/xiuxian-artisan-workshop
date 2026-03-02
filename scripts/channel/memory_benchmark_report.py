#!/usr/bin/env python3
"""Markdown report rendering helpers for memory benchmark runs."""

from __future__ import annotations

from typing import Any

from memory_benchmark_report_sections import (
    build_comparison_lines,
    build_confidence_lines,
    build_mode_summary_lines,
    build_run_metadata_lines,
    build_scenario_lines,
)


def format_float(value: float) -> str:
    """Format a float with a stable 4-decimal representation."""
    return f"{value:.4f}"


def build_markdown_report(
    *,
    config: Any,
    scenarios: tuple[Any, ...] | list[Any],
    started_at: str,
    finished_at: str,
    mode_summaries: dict[str, Any],
    comparison: dict[str, float] | None,
) -> str:
    """Render benchmark markdown report from run metadata and summaries."""
    lines: list[str] = ["# Omni-Agent Memory A/B Benchmark", ""]
    lines.extend(
        build_run_metadata_lines(
            config=config,
            scenarios=scenarios,
            started_at=started_at,
            finished_at=finished_at,
        )
    )
    lines.extend(build_scenario_lines(scenarios=scenarios))
    lines.extend(
        build_mode_summary_lines(
            config=config,
            mode_summaries=mode_summaries,
            format_float_fn=format_float,
        )
    )
    lines.extend(build_comparison_lines(comparison=comparison, format_float_fn=format_float))
    lines.extend(build_confidence_lines(mode_summaries=mode_summaries))
    return "\n".join(lines)
