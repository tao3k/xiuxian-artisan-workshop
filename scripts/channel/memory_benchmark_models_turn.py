#!/usr/bin/env python3
"""Turn-level telemetry datamodel for memory benchmark runner."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class TurnResult:
    """Structured turn-level benchmark telemetry."""

    mode: str
    iteration: int
    scenario_id: str
    query_index: int
    prompt: str
    expected_keywords: tuple[str, ...]
    required_ratio: float
    keyword_hit_ratio: float | None
    keyword_success: bool | None
    decision: str | None
    query_tokens: int | None
    recalled_selected: int | None
    recalled_injected: int | None
    context_chars_injected: int | None
    pipeline_duration_ms: int | None
    best_score: float | None
    weakest_score: float | None
    k1: int | None
    k2: int | None
    lambda_value: float | None
    min_score: float | None
    budget_pressure: float | None
    window_pressure: float | None
    recall_feedback_bias: float | None
    feedback_direction: str | None
    feedback_bias_before: float | None
    feedback_bias_after: float | None
    embedding_timeout_fallback_seen: bool = False
    embedding_cooldown_fallback_seen: bool = False
    embedding_unavailable_fallback_seen: bool = False
    mcp_error_detected: bool = False
    bot_excerpt: str | None = None
