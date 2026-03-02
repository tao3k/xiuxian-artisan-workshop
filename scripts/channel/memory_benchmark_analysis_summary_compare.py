#!/usr/bin/env python3
"""Summary comparison helpers for memory benchmark analysis."""

from __future__ import annotations

from typing import Any


def compare_mode_summaries(baseline: Any, adaptive: Any) -> dict[str, float]:
    """Compute adaptive-baseline deltas for summary metrics."""
    return {
        "success_rate_delta": adaptive.success_rate - baseline.success_rate,
        "avg_keyword_hit_ratio_delta": adaptive.avg_keyword_hit_ratio
        - baseline.avg_keyword_hit_ratio,
        "injected_rate_delta": adaptive.injected_rate - baseline.injected_rate,
        "avg_pipeline_duration_ms_delta": adaptive.avg_pipeline_duration_ms
        - baseline.avg_pipeline_duration_ms,
        "avg_recalled_selected_delta": adaptive.avg_recalled_selected
        - baseline.avg_recalled_selected,
        "avg_recalled_injected_delta": adaptive.avg_recalled_injected
        - baseline.avg_recalled_injected,
        "avg_best_score_delta": adaptive.avg_best_score - baseline.avg_best_score,
        "avg_recall_feedback_bias_delta": adaptive.avg_recall_feedback_bias
        - baseline.avg_recall_feedback_bias,
        "mcp_error_turns_delta": float(adaptive.mcp_error_turns - baseline.mcp_error_turns),
        "embedding_timeout_fallback_turns_delta": float(
            adaptive.embedding_timeout_fallback_turns - baseline.embedding_timeout_fallback_turns
        ),
        "embedding_cooldown_fallback_turns_delta": float(
            adaptive.embedding_cooldown_fallback_turns - baseline.embedding_cooldown_fallback_turns
        ),
        "embedding_unavailable_fallback_turns_delta": float(
            adaptive.embedding_unavailable_fallback_turns
            - baseline.embedding_unavailable_fallback_turns
        ),
        "embedding_fallback_turns_total_delta": float(
            adaptive.embedding_fallback_turns_total - baseline.embedding_fallback_turns_total
        ),
    }
