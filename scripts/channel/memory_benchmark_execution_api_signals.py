#!/usr/bin/env python3
"""Signal parsing APIs for memory benchmark execution."""

from __future__ import annotations

import importlib
from typing import Any

_signal_module = importlib.import_module("memory_benchmark_execution_signals")


def parse_turn_signals(
    lines: list[str],
    *,
    parse_turn_signals_fn: Any,
    forbidden_log_pattern: str,
    bot_marker: str,
    recall_plan_event: str,
    recall_injected_event: str,
    recall_skipped_event: str,
    recall_feedback_event: str,
    embedding_timeout_fallback_event: str,
    embedding_cooldown_fallback_event: str,
    embedding_unavailable_fallback_event: str,
) -> dict[str, Any]:
    """Parse observability signals from runtime log lines."""
    return _signal_module.parse_turn_signals(
        lines,
        parse_turn_signals_fn=parse_turn_signals_fn,
        forbidden_log_pattern=forbidden_log_pattern,
        bot_marker=bot_marker,
        recall_plan_event=recall_plan_event,
        recall_injected_event=recall_injected_event,
        recall_skipped_event=recall_skipped_event,
        recall_feedback_event=recall_feedback_event,
        embedding_timeout_fallback_event=embedding_timeout_fallback_event,
        embedding_cooldown_fallback_event=embedding_cooldown_fallback_event,
        embedding_unavailable_fallback_event=embedding_unavailable_fallback_event,
    )


def build_turn_result(
    *,
    mode: str,
    iteration: int,
    scenario_id: str,
    query_index: int,
    query: Any,
    lines: list[str],
    feedback_direction: str | None = None,
    feedback_lines: list[str] | None = None,
    parse_turn_signals_fn: Any,
    keyword_hit_ratio_fn: Any,
    token_as_int_fn: Any,
    token_as_float_fn: Any,
    trim_text_fn: Any,
    turn_result_cls: Any,
) -> Any:
    """Build one structured benchmark turn result."""
    return _signal_module.build_turn_result(
        mode=mode,
        iteration=iteration,
        scenario_id=scenario_id,
        query_index=query_index,
        query=query,
        lines=lines,
        feedback_direction=feedback_direction,
        feedback_lines=feedback_lines,
        parse_turn_signals_fn=parse_turn_signals_fn,
        keyword_hit_ratio_fn=keyword_hit_ratio_fn,
        token_as_int_fn=token_as_int_fn,
        token_as_float_fn=token_as_float_fn,
        trim_text_fn=trim_text_fn,
        turn_result_cls=turn_result_cls,
    )


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
    return _signal_module.summarize_mode(
        mode=mode,
        iterations=iterations,
        scenario_count=scenario_count,
        turns=turns,
        summarize_mode_data_fn=summarize_mode_data_fn,
        mode_summary_cls=mode_summary_cls,
    )
