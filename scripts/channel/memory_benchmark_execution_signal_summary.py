#!/usr/bin/env python3
"""Summary helpers for memory benchmark execution."""

from __future__ import annotations

from typing import Any


def summarize_mode(
    *,
    mode: str,
    iterations: int,
    scenario_count: int,
    turns: list[Any],
    summarize_mode_data_fn: Any,
    mode_summary_cls: Any,
) -> Any:
    """Aggregate per-mode benchmark summary."""
    payload = summarize_mode_data_fn(
        mode=mode,
        iterations=iterations,
        scenario_count=scenario_count,
        turns=turns,
    )
    return mode_summary_cls(**payload)
