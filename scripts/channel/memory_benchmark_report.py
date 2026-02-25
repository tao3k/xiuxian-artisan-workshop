#!/usr/bin/env python3
"""Markdown report rendering helpers for memory benchmark runs."""

from __future__ import annotations

from typing import Any


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
    lines: list[str] = []
    lines.append("# Omni-Agent Memory A/B Benchmark")
    lines.append("")
    lines.append("## Run Metadata")
    lines.append("")
    lines.append(f"- started_at_utc: `{started_at}`")
    lines.append(f"- finished_at_utc: `{finished_at}`")
    lines.append(f"- dataset: `{config.dataset_path}`")
    lines.append(f"- log_file: `{config.log_file}`")
    lines.append(
        f"- session_target: `chat={config.chat_id}, user={config.user_id}, "
        f"thread={config.thread_id if config.thread_id is not None else 'none'}`"
    )
    lines.append(f"- runtime_partition_mode: `{config.runtime_partition_mode or 'unknown'}`")
    lines.append(f"- modes: `{', '.join(config.modes)}`")
    lines.append(f"- iterations_per_mode: `{config.iterations}`")
    lines.append(f"- scenario_count: `{len(scenarios)}`")
    lines.append(f"- feedback_policy: `{config.feedback_policy}`")
    lines.append(f"- feedback_down_threshold: `{config.feedback_down_threshold}`")
    lines.append("")

    lines.append("## Scenario Set")
    lines.append("")
    for scenario in scenarios:
        lines.append(
            f"- `{scenario.scenario_id}`: {scenario.description} "
            f"(setup={len(scenario.setup_prompts)}, queries={len(scenario.queries)})"
        )
    lines.append("")

    lines.append("## Mode Summary")
    lines.append("")
    lines.append(
        "| Mode | Query Turns | Scored Turns | Success Rate | Avg Hit Ratio | Injected Rate | Avg Pipeline ms | Avg k1 | Avg k2 | Avg lambda | Avg Feedback Bias | MCP Error Turns | Embedding Fallback Turns |"
    )
    lines.append(
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    )
    for mode in config.modes:
        summary = mode_summaries[mode]
        lines.append(
            "| "
            f"{mode} | "
            f"{summary.query_turns} | "
            f"{summary.scored_turns} | "
            f"{format_float(summary.success_rate)} | "
            f"{format_float(summary.avg_keyword_hit_ratio)} | "
            f"{format_float(summary.injected_rate)} | "
            f"{format_float(summary.avg_pipeline_duration_ms)} | "
            f"{format_float(summary.avg_k1)} | "
            f"{format_float(summary.avg_k2)} | "
            f"{format_float(summary.avg_lambda)} | "
            f"{format_float(summary.avg_recall_feedback_bias)} | "
            f"{summary.mcp_error_turns} | "
            f"{summary.embedding_fallback_turns_total} |"
        )
    lines.append("")

    if comparison is not None:
        lines.append("## Adaptive Delta vs Baseline")
        lines.append("")
        lines.append("| Metric | Delta |")
        lines.append("| --- | ---: |")
        for key, value in comparison.items():
            lines.append(f"| {key} | {format_float(value)} |")
        lines.append("")

    confidence_note = "moderate"
    baseline = mode_summaries.get("baseline")
    adaptive = mode_summaries.get("adaptive")
    if baseline and adaptive and min(baseline.scored_turns, adaptive.scored_turns) < 5:
        confidence_note = "low"
    total_mcp_error_turns = sum(summary.mcp_error_turns for summary in mode_summaries.values())
    total_embedding_fallback_turns = sum(
        summary.embedding_fallback_turns_total for summary in mode_summaries.values()
    )

    lines.append("## Confidence")
    lines.append("")
    lines.append(
        "- This benchmark is proxy-based (keyword hit + memory observability metrics), "
        "not a human-graded semantic evaluation."
    )
    lines.append(
        f"- Confidence level for this run: `{confidence_note}` "
        f"(scored turns may be small; increase `--iterations` for stronger signal)."
    )
    if total_mcp_error_turns > 0:
        lines.append(
            f"- MCP error interference observed on `{total_mcp_error_turns}` query turn(s)."
        )
    if total_embedding_fallback_turns > 0:
        lines.append(
            f"- Embedding fallback observed on `{total_embedding_fallback_turns}` query turn(s)."
        )
    lines.append("")

    return "\n".join(lines)
