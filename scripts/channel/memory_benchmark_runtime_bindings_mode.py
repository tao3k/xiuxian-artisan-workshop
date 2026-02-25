#!/usr/bin/env python3
"""Mode aggregation bindings for memory benchmark runtime entrypoint."""

from __future__ import annotations

from typing import Any


def summarize_mode(
    *,
    mode: str,
    iterations: int,
    scenario_count: int,
    turns: list[Any],
    execution_module: Any,
    summarize_mode_data_fn: Any,
    mode_summary_cls: Any,
) -> Any:
    """Aggregate per-mode benchmark summary."""
    return execution_module.summarize_mode(
        mode=mode,
        iterations=iterations,
        scenario_count=scenario_count,
        turns=turns,
        summarize_mode_data_fn=summarize_mode_data_fn,
        mode_summary_cls=mode_summary_cls,
    )


def run_mode(
    config: Any,
    scenarios: tuple[Any, ...],
    mode: str,
    *,
    execution_module: Any,
    run_reset_fn: Any,
    run_non_command_turn_fn: Any,
    build_turn_result_fn: Any,
    select_feedback_direction_fn: Any,
    run_feedback_fn: Any,
) -> list[Any]:
    """Run all scenarios for one benchmark mode."""
    return execution_module.run_mode(
        config,
        scenarios,
        mode,
        run_reset_fn=run_reset_fn,
        run_non_command_turn_fn=run_non_command_turn_fn,
        build_turn_result_fn=build_turn_result_fn,
        select_feedback_direction_fn=select_feedback_direction_fn,
        run_feedback_fn=run_feedback_fn,
    )
