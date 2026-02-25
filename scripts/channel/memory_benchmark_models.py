#!/usr/bin/env python3
"""Datamodels for memory benchmark runner."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class QuerySpec:
    """One benchmark query and its keyword quality target."""

    prompt: str
    expected_keywords: tuple[str, ...]
    required_ratio: float


@dataclass(frozen=True)
class ScenarioSpec:
    """One benchmark scenario containing setup prompts and query turns."""

    scenario_id: str
    description: str
    setup_prompts: tuple[str, ...]
    queries: tuple[QuerySpec, ...]
    reset_before: bool = True
    reset_after: bool = False


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


@dataclass
class BenchmarkConfig:
    """Runtime configuration for memory benchmark execution."""

    dataset_path: Path
    log_file: Path
    blackbox_script: Path
    chat_id: int
    user_id: int
    thread_id: int | None
    runtime_partition_mode: str | None
    username: str
    max_wait: int
    max_idle_secs: int
    modes: tuple[str, ...]
    iterations: int
    skip_reset: bool
    output_json: Path
    output_markdown: Path
    fail_on_mcp_error: bool
    feedback_policy: str
    feedback_down_threshold: float
