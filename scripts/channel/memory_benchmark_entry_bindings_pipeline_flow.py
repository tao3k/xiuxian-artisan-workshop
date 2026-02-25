#!/usr/bin/env python3
"""Flow passthrough bindings for memory benchmark entrypoint."""

from __future__ import annotations

from typing import Any


def run_reset(
    config: Any,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    run_probe_fn: Any,
    reset_event: str,
) -> None:
    """Run reset command turn."""
    runtime_bindings_module.run_reset(
        config,
        execution_module=execution_module,
        run_probe_fn=run_probe_fn,
        reset_event=reset_event,
    )


def run_feedback(
    config: Any,
    direction: str,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    run_probe_fn: Any,
    feedback_event: str,
) -> list[str]:
    """Run feedback command turn."""
    return runtime_bindings_module.run_feedback(
        config,
        direction,
        execution_module=execution_module,
        run_probe_fn=run_probe_fn,
        feedback_event=feedback_event,
    )


def run_non_command_turn(
    config: Any,
    prompt: str,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    run_probe_fn: Any,
    recall_plan_event: str,
) -> list[str]:
    """Run a regular query turn."""
    return runtime_bindings_module.run_non_command_turn(
        config,
        prompt,
        execution_module=execution_module,
        run_probe_fn=run_probe_fn,
        recall_plan_event=recall_plan_event,
    )


def run_mode(
    config: Any,
    scenarios: tuple[Any, ...],
    mode: str,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    run_reset_fn: Any,
    run_non_command_turn_fn: Any,
    build_turn_result_fn: Any,
    select_feedback_direction_fn: Any,
    run_feedback_fn: Any,
) -> list[Any]:
    """Run all scenarios for one mode."""
    return runtime_bindings_module.run_mode(
        config,
        scenarios,
        mode,
        execution_module=execution_module,
        run_reset_fn=run_reset_fn,
        run_non_command_turn_fn=run_non_command_turn_fn,
        build_turn_result_fn=build_turn_result_fn,
        select_feedback_direction_fn=select_feedback_direction_fn,
        run_feedback_fn=run_feedback_fn,
    )
