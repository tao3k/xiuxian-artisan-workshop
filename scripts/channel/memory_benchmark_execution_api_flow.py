#!/usr/bin/env python3
"""Mode-flow API for memory benchmark execution."""

from __future__ import annotations

import importlib
from typing import Any

_flow_module = importlib.import_module("memory_benchmark_execution_flow")


def run_mode(
    config: Any,
    scenarios: tuple[Any, ...],
    mode: str,
    *,
    run_reset_fn: Any,
    run_non_command_turn_fn: Any,
    build_turn_result_fn: Any,
    select_feedback_direction_fn: Any,
    run_feedback_fn: Any,
) -> list[Any]:
    """Run all scenarios for one benchmark mode."""
    return _flow_module.run_mode(
        config,
        scenarios,
        mode,
        run_reset_fn=run_reset_fn,
        run_non_command_turn_fn=run_non_command_turn_fn,
        build_turn_result_fn=build_turn_result_fn,
        select_feedback_direction_fn=select_feedback_direction_fn,
        run_feedback_fn=run_feedback_fn,
    )
