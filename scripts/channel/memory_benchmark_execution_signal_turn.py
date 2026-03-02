#!/usr/bin/env python3
"""Turn-result construction helpers for memory benchmark execution."""

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
    signals = parse_turn_signals_fn(lines)
    plan = signals.get("plan") or {}
    decision = signals.get("decision") or {}

    bot_line = signals.get("bot_line")
    hit_ratio = keyword_hit_ratio_fn(bot_line, query.expected_keywords)
    success = None
    if hit_ratio is not None:
        success = hit_ratio >= query.required_ratio

    feedback_before: float | None = None
    feedback_after: float | None = None
    if feedback_lines:
        feedback_signals = parse_turn_signals_fn(feedback_lines)
        feedback_tokens = feedback_signals.get("feedback") or {}
        feedback_before = token_as_float_fn(feedback_tokens, "recall_feedback_bias_before")
        feedback_after = token_as_float_fn(feedback_tokens, "recall_feedback_bias_after")

    return turn_result_cls(
        mode=mode,
        iteration=iteration,
        scenario_id=scenario_id,
        query_index=query_index,
        prompt=query.prompt,
        expected_keywords=query.expected_keywords,
        required_ratio=query.required_ratio,
        keyword_hit_ratio=hit_ratio,
        keyword_success=success,
        decision=(decision.get("event") or "").split(".")[-1] or None,
        query_tokens=token_as_int_fn(decision, "query_tokens"),
        recalled_selected=token_as_int_fn(decision, "recalled_selected"),
        recalled_injected=token_as_int_fn(decision, "recalled_injected"),
        context_chars_injected=token_as_int_fn(decision, "context_chars_injected"),
        pipeline_duration_ms=token_as_int_fn(decision, "pipeline_duration_ms"),
        best_score=token_as_float_fn(decision, "best_score"),
        weakest_score=token_as_float_fn(decision, "weakest_score"),
        k1=token_as_int_fn(plan, "k1"),
        k2=token_as_int_fn(plan, "k2"),
        lambda_value=token_as_float_fn(plan, "lambda"),
        min_score=token_as_float_fn(plan, "min_score"),
        budget_pressure=token_as_float_fn(plan, "budget_pressure"),
        window_pressure=token_as_float_fn(plan, "window_pressure"),
        recall_feedback_bias=token_as_float_fn(plan, "recall_feedback_bias"),
        feedback_direction=feedback_direction,
        feedback_bias_before=feedback_before,
        feedback_bias_after=feedback_after,
        embedding_timeout_fallback_seen=bool(signals.get("embedding_timeout_fallback")),
        embedding_cooldown_fallback_seen=bool(signals.get("embedding_cooldown_fallback")),
        embedding_unavailable_fallback_seen=bool(signals.get("embedding_unavailable_fallback")),
        mcp_error_detected=bool(signals.get("mcp_error")),
        bot_excerpt=trim_text_fn(bot_line),
    )
