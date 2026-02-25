#!/usr/bin/env python3
"""Command-turn bindings for memory benchmark runtime entrypoint."""

from __future__ import annotations

from typing import Any


def run_reset(
    config: Any,
    *,
    execution_module: Any,
    run_probe_fn: Any,
    reset_event: str,
) -> None:
    """Execute reset command probe."""
    execution_module.run_reset(
        config,
        run_probe_fn=run_probe_fn,
        reset_event=reset_event,
    )


def run_feedback(
    config: Any,
    direction: str,
    *,
    execution_module: Any,
    run_probe_fn: Any,
    feedback_event: str,
) -> list[str]:
    """Execute adaptive feedback command probe."""
    return execution_module.run_feedback(
        config,
        direction,
        run_probe_fn=run_probe_fn,
        feedback_event=feedback_event,
    )


def run_non_command_turn(
    config: Any,
    prompt: str,
    *,
    execution_module: Any,
    run_probe_fn: Any,
    recall_plan_event: str,
) -> list[str]:
    """Execute one regular non-command prompt turn."""
    return execution_module.run_non_command_turn(
        config,
        prompt,
        run_probe_fn=run_probe_fn,
        recall_plan_event=recall_plan_event,
    )
