#!/usr/bin/env python3
"""Compatibility facade for memory benchmark execution signal helpers."""

from __future__ import annotations

from typing import Any

from memory_benchmark_execution_signal_parse import parse_turn_signals as _parse_turn_signals_impl
from memory_benchmark_execution_signal_summary import summarize_mode as _summarize_mode_impl
from memory_benchmark_execution_signal_turn import build_turn_result as _build_turn_result_impl


def parse_turn_signals(lines: list[str], **kwargs: Any) -> dict[str, Any]:
    """Parse observability signals from runtime log lines."""
    return _parse_turn_signals_impl(lines, **kwargs)


def build_turn_result(**kwargs: Any) -> Any:
    """Build one structured benchmark turn result."""
    return _build_turn_result_impl(**kwargs)


def summarize_mode(**kwargs: Any) -> Any:
    """Aggregate per-mode benchmark summary."""
    return _summarize_mode_impl(**kwargs)
