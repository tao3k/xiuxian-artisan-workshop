#!/usr/bin/env python3
"""Mode summary passthrough for memory benchmark entry bindings."""

from __future__ import annotations

from typing import Any


def summarize_mode(
    *,
    mode: str,
    iterations: int,
    scenario_count: int,
    turns: list[Any],
    runtime_bindings_module: Any,
    execution_module: Any,
    summarize_mode_data_fn: Any,
    mode_summary_cls: Any,
) -> Any:
    """Summarize one benchmark mode."""
    return runtime_bindings_module.summarize_mode(
        mode=mode,
        iterations=iterations,
        scenario_count=scenario_count,
        turns=turns,
        execution_module=execution_module,
        summarize_mode_data_fn=summarize_mode_data_fn,
        mode_summary_cls=mode_summary_cls,
    )
