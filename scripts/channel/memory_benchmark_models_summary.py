#!/usr/bin/env python3
"""Mode-level summary datamodel for memory benchmark runner."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ModeSummary:
    """Mode-level aggregate benchmark summary."""

    mode: str
    iterations: int
    scenarios: int
    query_turns: int
    scored_turns: int
    success_count: int
    success_rate: float
    avg_keyword_hit_ratio: float
    injected_count: int
    skipped_count: int
    injected_rate: float
    avg_pipeline_duration_ms: float
    avg_query_tokens: float
    avg_recalled_selected: float
    avg_recalled_injected: float
    avg_context_chars_injected: float
    avg_best_score: float
    avg_weakest_score: float
    avg_k1: float
    avg_k2: float
    avg_lambda: float
    avg_min_score: float
    avg_budget_pressure: float
    avg_window_pressure: float
    avg_recall_feedback_bias: float
    feedback_updates: int
    feedback_up_count: int
    feedback_down_count: int
    avg_feedback_delta: float
    mcp_error_turns: int
    embedding_timeout_fallback_turns: int
    embedding_cooldown_fallback_turns: int
    embedding_unavailable_fallback_turns: int
    embedding_fallback_turns_total: int
