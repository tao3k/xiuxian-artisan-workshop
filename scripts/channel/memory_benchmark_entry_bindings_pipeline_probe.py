#!/usr/bin/env python3
"""Probe/signal passthrough bindings for memory benchmark entrypoint."""

from __future__ import annotations

from typing import Any


def run_probe(
    config: Any,
    *,
    prompt: str,
    expect_event: str,
    allow_no_bot: bool = False,
    runtime_bindings_module: Any,
    execution_module: Any,
    count_lines_fn: Any,
    read_new_lines_fn: Any,
    strip_ansi_fn: Any,
    has_event_fn: Any,
    control_admin_required_event: str,
    forbidden_log_pattern: str,
) -> list[str]:
    """Run one probe turn through runtime bindings."""
    return runtime_bindings_module.run_probe(
        config,
        prompt=prompt,
        expect_event=expect_event,
        allow_no_bot=allow_no_bot,
        execution_module=execution_module,
        count_lines_fn=count_lines_fn,
        read_new_lines_fn=read_new_lines_fn,
        strip_ansi_fn=strip_ansi_fn,
        has_event_fn=has_event_fn,
        control_admin_required_event=control_admin_required_event,
        forbidden_log_pattern=forbidden_log_pattern,
    )


def parse_turn_signals(
    lines: list[str],
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
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
    """Parse memory observability signals for one turn."""
    return runtime_bindings_module.parse_turn_signals(
        lines,
        execution_module=execution_module,
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
