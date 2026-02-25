#!/usr/bin/env python3
"""Datamodels and errors for memory CI gate runner."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class GateConfig:
    profile: str
    project_root: Path
    script_dir: Path
    agent_bin: Path | None
    webhook_port: int
    telegram_api_port: int
    valkey_port: int
    valkey_url: str
    valkey_prefix: str
    username: str
    webhook_secret: str
    chat_id: int
    chat_b: int
    chat_c: int
    user_id: int
    user_b: int
    user_c: int
    runtime_log_file: Path
    mock_log_file: Path
    runtime_startup_timeout_secs: int
    quick_max_wait: int
    quick_max_idle: int
    full_max_wait: int
    full_max_idle: int
    matrix_max_wait: int
    matrix_max_idle: int
    benchmark_iterations: int
    skip_matrix: bool
    skip_benchmark: bool
    skip_evolution: bool
    skip_rust_regressions: bool
    skip_discover_cache_gate: bool
    skip_reflection_quality_gate: bool
    skip_trace_reconstruction_gate: bool
    skip_cross_group_complex_gate: bool
    evolution_report_json: Path
    benchmark_report_json: Path
    session_matrix_report_json: Path
    session_matrix_report_markdown: Path
    trace_report_json: Path
    trace_report_markdown: Path
    cross_group_report_json: Path
    cross_group_report_markdown: Path
    cross_group_dataset: Path
    cross_group_scenario_id: str
    min_planned_hits: int
    min_successful_corrections: int
    min_recall_credit_events: int
    min_quality_score: float
    slow_response_min_duration_ms: int
    slow_response_long_step_ms: int
    slow_response_min_long_steps: int
    trace_min_quality_score: float
    trace_max_events: int
    min_session_steps: int
    require_cross_group_step: bool
    require_mixed_batch_steps: bool
    cross_group_max_wait: int
    cross_group_max_idle: int
    cross_group_max_parallel: int
    discover_cache_hit_p95_ms: float
    discover_cache_miss_p95_ms: float
    discover_cache_bench_iterations: int
    max_mcp_call_waiting_events: int
    max_mcp_connect_waiting_events: int
    max_mcp_waiting_events_total: int
    max_memory_stream_read_failed_events: int
    max_embedding_timeout_fallback_turns: int
    max_embedding_cooldown_fallback_turns: int
    max_embedding_unavailable_fallback_turns: int
    max_embedding_fallback_turns_total: int


class GateStepError(RuntimeError):
    """Raised when a named gate subprocess command fails."""

    def __init__(self, *, title: str, cmd: list[str], returncode: int) -> None:
        self.title = title
        self.cmd = list(cmd)
        self.returncode = int(returncode)
        super().__init__(f"{title} failed (exit={self.returncode})")
