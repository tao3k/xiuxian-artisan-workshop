#!/usr/bin/env python3
"""Output builders for memory benchmark reports."""

from __future__ import annotations

import json
from dataclasses import asdict
from typing import Any


def serialize_turn(turn: Any) -> dict[str, Any]:
    """Serialize turn result dataclass to JSON-friendly dict."""
    payload = asdict(turn)
    payload["expected_keywords"] = list(turn.expected_keywords)
    return payload


def build_json_payload(
    *,
    config: Any,
    scenarios: tuple[Any, ...],
    mode_summaries: dict[str, Any],
    comparison: dict[str, float] | None,
    mode_turns: dict[str, list[Any]],
    started_at: str,
    finished_at: str,
    duration_secs: float,
) -> dict[str, Any]:
    """Build machine-readable benchmark payload."""
    return {
        "metadata": {
            "started_at_utc": started_at,
            "finished_at_utc": finished_at,
            "duration_secs": round(duration_secs, 3),
            "dataset": str(config.dataset_path),
            "log_file": str(config.log_file),
            "chat_id": config.chat_id,
            "user_id": config.user_id,
            "thread_id": config.thread_id,
            "runtime_partition_mode": config.runtime_partition_mode,
            "modes": list(config.modes),
            "iterations_per_mode": config.iterations,
            "scenario_count": len(scenarios),
            "max_wait_secs": config.max_wait,
            "max_idle_secs": config.max_idle_secs,
            "username": config.username,
            "skip_reset": config.skip_reset,
            "fail_on_mcp_error": config.fail_on_mcp_error,
            "feedback_policy": config.feedback_policy,
            "feedback_down_threshold": config.feedback_down_threshold,
        },
        "scenarios": [
            {
                "id": scenario.scenario_id,
                "description": scenario.description,
                "setup_prompts": list(scenario.setup_prompts),
                "queries": [
                    {
                        "prompt": query.prompt,
                        "expected_keywords": list(query.expected_keywords),
                        "required_ratio": query.required_ratio,
                    }
                    for query in scenario.queries
                ],
            }
            for scenario in scenarios
        ],
        "mode_summaries": {mode: asdict(summary) for mode, summary in mode_summaries.items()},
        "comparison": comparison,
        "turns": {
            mode: [serialize_turn(turn) for turn in turns] for mode, turns in mode_turns.items()
        },
    }


def write_outputs(*, config: Any, json_payload: dict[str, Any], markdown: str) -> None:
    """Write JSON and Markdown benchmark reports to configured paths."""
    config.output_json.write_text(
        json.dumps(json_payload, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    config.output_markdown.write_text(markdown + "\n", encoding="utf-8")


def print_summary(
    *,
    config: Any,
    mode_summaries: dict[str, Any],
    comparison: dict[str, float] | None,
) -> None:
    """Print benchmark completion summary to stdout."""
    print("\nBenchmark completed.", flush=True)
    print(f"JSON report: {config.output_json}", flush=True)
    print(f"Markdown report: {config.output_markdown}", flush=True)
    total_mcp_error_turns = sum(summary.mcp_error_turns for summary in mode_summaries.values())
    if total_mcp_error_turns > 0:
        print(
            f"Observed MCP error interference on {total_mcp_error_turns} query turn(s).",
            flush=True,
        )
    if comparison is not None:
        print("Adaptive delta vs baseline:", flush=True)
        for key, value in comparison.items():
            print(f"  {key}={value:.4f}", flush=True)
