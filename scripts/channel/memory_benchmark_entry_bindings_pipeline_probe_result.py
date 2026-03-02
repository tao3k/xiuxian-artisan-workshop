#!/usr/bin/env python3
"""Turn-result builder passthrough for memory benchmark entry bindings."""

from __future__ import annotations

from typing import Any


def build_turn_result(
    *,
    mode: str,
    iteration: int,
    scenario_id: str,
    query_index: int,
    query: Any,
    lines: list[str],
    feedback_direction: str | None,
    feedback_lines: list[str] | None,
    runtime_bindings_module: Any,
    execution_module: Any,
    parse_turn_signals_fn: Any,
    keyword_hit_ratio_fn: Any,
    token_as_int_fn: Any,
    token_as_float_fn: Any,
    trim_text_fn: Any,
    turn_result_cls: Any,
) -> Any:
    """Build structured benchmark turn result."""
    return runtime_bindings_module.build_turn_result(
        mode=mode,
        iteration=iteration,
        scenario_id=scenario_id,
        query_index=query_index,
        query=query,
        lines=lines,
        feedback_direction=feedback_direction,
        feedback_lines=feedback_lines,
        execution_module=execution_module,
        parse_turn_signals_fn=parse_turn_signals_fn,
        keyword_hit_ratio_fn=keyword_hit_ratio_fn,
        token_as_int_fn=token_as_int_fn,
        token_as_float_fn=token_as_float_fn,
        trim_text_fn=trim_text_fn,
        turn_result_cls=turn_result_cls,
    )
