#!/usr/bin/env python3
"""Signal parsing passthrough for memory benchmark entry bindings."""

from __future__ import annotations

from typing import Any


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
