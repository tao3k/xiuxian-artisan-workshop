#!/usr/bin/env python3
"""Section builders for memory benchmark markdown reports."""

from __future__ import annotations

from typing import Any

from memory_benchmark_report_sections_confidence import (
    build_confidence_lines as _build_confidence_lines_impl,
)


def build_run_metadata_lines(
    *,
    config: Any,
    scenarios: tuple[Any, ...] | list[Any],
    started_at: str,
    finished_at: str,
) -> list[str]:
    """Build run-metadata section lines."""
    return [
        "## Run Metadata",
        "",
        f"- started_at_utc: `{started_at}`",
        f"- finished_at_utc: `{finished_at}`",
        f"- dataset: `{config.dataset_path}`",
        f"- log_file: `{config.log_file}`",
        f"- session_target: `chat={config.chat_id}, user={config.user_id}, "
        f"thread={config.thread_id if config.thread_id is not None else 'none'}`",
        f"- runtime_partition_mode: `{config.runtime_partition_mode or 'unknown'}`",
        f"- modes: `{', '.join(config.modes)}`",
        f"- iterations_per_mode: `{config.iterations}`",
        f"- scenario_count: `{len(scenarios)}`",
        f"- feedback_policy: `{config.feedback_policy}`",
        f"- feedback_down_threshold: `{config.feedback_down_threshold}`",
        "",
    ]


def build_scenario_lines(*, scenarios: tuple[Any, ...] | list[Any]) -> list[str]:
    """Build scenario-list section lines."""
    lines = ["## Scenario Set", ""]
    for scenario in scenarios:
        lines.append(
            f"- `{scenario.scenario_id}`: {scenario.description} "
            f"(setup={len(scenario.setup_prompts)}, queries={len(scenario.queries)})"
        )
    lines.append("")
    return lines


def build_mode_summary_lines(
    *,
    config: Any,
    mode_summaries: dict[str, Any],
    format_float_fn: Any,
) -> list[str]:
    """Build mode-summary table lines."""
    lines = [
        "## Mode Summary",
        "",
        "| Mode | Query Turns | Scored Turns | Success Rate | Avg Hit Ratio | Injected Rate | Avg Pipeline ms | Avg k1 | Avg k2 | Avg lambda | Avg Feedback Bias | MCP Error Turns | Embedding Fallback Turns |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for mode in config.modes:
        summary = mode_summaries[mode]
        lines.append(
            "| "
            f"{mode} | "
            f"{summary.query_turns} | "
            f"{summary.scored_turns} | "
            f"{format_float_fn(summary.success_rate)} | "
            f"{format_float_fn(summary.avg_keyword_hit_ratio)} | "
            f"{format_float_fn(summary.injected_rate)} | "
            f"{format_float_fn(summary.avg_pipeline_duration_ms)} | "
            f"{format_float_fn(summary.avg_k1)} | "
            f"{format_float_fn(summary.avg_k2)} | "
            f"{format_float_fn(summary.avg_lambda)} | "
            f"{format_float_fn(summary.avg_recall_feedback_bias)} | "
            f"{summary.mcp_error_turns} | "
            f"{summary.embedding_fallback_turns_total} |"
        )
    lines.append("")
    return lines


def build_comparison_lines(
    *,
    comparison: dict[str, float] | None,
    format_float_fn: Any,
) -> list[str]:
    """Build adaptive-vs-baseline comparison section lines."""
    if comparison is None:
        return []
    lines = [
        "## Adaptive Delta vs Baseline",
        "",
        "| Metric | Delta |",
        "| --- | ---: |",
    ]
    for key, value in comparison.items():
        lines.append(f"| {key} | {format_float_fn(value)} |")
    lines.append("")
    return lines


def build_confidence_lines(*, mode_summaries: dict[str, Any]) -> list[str]:
    """Build confidence section lines."""
    return _build_confidence_lines_impl(mode_summaries=mode_summaries)
