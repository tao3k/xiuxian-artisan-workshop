#!/usr/bin/env python3
"""Summary aggregation helpers for memory benchmark analysis."""

from __future__ import annotations

from typing import Any

from memory_benchmark_analysis_stats import maybe_mean, maybe_mean_int


def summarize_mode_data(
    *,
    mode: str,
    iterations: int,
    scenario_count: int,
    turns: list[Any],
) -> dict[str, Any]:
    """Aggregate per-turn metrics into one mode summary payload."""
    scored_turns = [turn for turn in turns if turn.keyword_hit_ratio is not None]
    successful_turns = [turn for turn in scored_turns if turn.keyword_success]

    query_tokens = [value for turn in turns if (value := turn.query_tokens) is not None]
    recalled_selected = [value for turn in turns if (value := turn.recalled_selected) is not None]
    recalled_injected = [value for turn in turns if (value := turn.recalled_injected) is not None]
    context_chars = [value for turn in turns if (value := turn.context_chars_injected) is not None]
    pipeline_duration = [
        value for turn in turns if (value := turn.pipeline_duration_ms) is not None
    ]
    best_scores = [value for turn in turns if (value := turn.best_score) is not None]
    weakest_scores = [value for turn in turns if (value := turn.weakest_score) is not None]

    k1_values = [value for turn in turns if (value := turn.k1) is not None]
    k2_values = [value for turn in turns if (value := turn.k2) is not None]
    lambda_values = [value for turn in turns if (value := turn.lambda_value) is not None]
    min_score_values = [value for turn in turns if (value := turn.min_score) is not None]
    budget_pressure_values = [
        value for turn in turns if (value := turn.budget_pressure) is not None
    ]
    window_pressure_values = [
        value for turn in turns if (value := turn.window_pressure) is not None
    ]
    recall_bias_values = [
        value for turn in turns if (value := turn.recall_feedback_bias) is not None
    ]

    feedback_updates = [
        turn
        for turn in turns
        if turn.feedback_bias_before is not None and turn.feedback_bias_after is not None
    ]
    feedback_deltas = [
        turn.feedback_bias_after - turn.feedback_bias_before for turn in feedback_updates
    ]
    feedback_up_count = sum(1 for turn in turns if turn.feedback_direction == "up")
    feedback_down_count = sum(1 for turn in turns if turn.feedback_direction == "down")
    mcp_error_turns = sum(1 for turn in turns if turn.mcp_error_detected)
    embedding_timeout_fallback_turns = sum(
        1 for turn in turns if turn.embedding_timeout_fallback_seen
    )
    embedding_cooldown_fallback_turns = sum(
        1 for turn in turns if turn.embedding_cooldown_fallback_seen
    )
    embedding_unavailable_fallback_turns = sum(
        1 for turn in turns if turn.embedding_unavailable_fallback_seen
    )
    embedding_fallback_turns_total = sum(
        1
        for turn in turns
        if turn.embedding_timeout_fallback_seen
        or turn.embedding_cooldown_fallback_seen
        or turn.embedding_unavailable_fallback_seen
    )

    injected_count = sum(1 for turn in turns if turn.decision == "injected")
    skipped_count = sum(1 for turn in turns if turn.decision == "skipped")
    completed_count = injected_count + skipped_count

    return {
        "mode": mode,
        "iterations": iterations,
        "scenarios": scenario_count,
        "query_turns": len(turns),
        "scored_turns": len(scored_turns),
        "success_count": len(successful_turns),
        "success_rate": (len(successful_turns) / len(scored_turns)) if scored_turns else 0.0,
        "avg_keyword_hit_ratio": maybe_mean(
            [value for turn in scored_turns if (value := turn.keyword_hit_ratio) is not None]
        ),
        "injected_count": injected_count,
        "skipped_count": skipped_count,
        "injected_rate": (injected_count / completed_count) if completed_count else 0.0,
        "avg_pipeline_duration_ms": maybe_mean_int(pipeline_duration),
        "avg_query_tokens": maybe_mean_int(query_tokens),
        "avg_recalled_selected": maybe_mean_int(recalled_selected),
        "avg_recalled_injected": maybe_mean_int(recalled_injected),
        "avg_context_chars_injected": maybe_mean_int(context_chars),
        "avg_best_score": maybe_mean(best_scores),
        "avg_weakest_score": maybe_mean(weakest_scores),
        "avg_k1": maybe_mean_int(k1_values),
        "avg_k2": maybe_mean_int(k2_values),
        "avg_lambda": maybe_mean(lambda_values),
        "avg_min_score": maybe_mean(min_score_values),
        "avg_budget_pressure": maybe_mean(budget_pressure_values),
        "avg_window_pressure": maybe_mean(window_pressure_values),
        "avg_recall_feedback_bias": maybe_mean(recall_bias_values),
        "feedback_updates": len(feedback_updates),
        "feedback_up_count": feedback_up_count,
        "feedback_down_count": feedback_down_count,
        "avg_feedback_delta": maybe_mean(feedback_deltas),
        "mcp_error_turns": mcp_error_turns,
        "embedding_timeout_fallback_turns": embedding_timeout_fallback_turns,
        "embedding_cooldown_fallback_turns": embedding_cooldown_fallback_turns,
        "embedding_unavailable_fallback_turns": embedding_unavailable_fallback_turns,
        "embedding_fallback_turns_total": embedding_fallback_turns_total,
    }


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
