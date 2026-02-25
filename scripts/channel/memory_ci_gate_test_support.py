#!/usr/bin/env python3
"""Shared test fixtures for memory CI gate test suites."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

from test_omni_agent_memory_ci_gate import GateConfig

if TYPE_CHECKING:
    from pathlib import Path


def build_cfg(tmp_path: Path) -> GateConfig:
    """Build baseline nightly GateConfig for tests."""
    return GateConfig(
        profile="nightly",
        project_root=tmp_path,
        script_dir=tmp_path,
        agent_bin=None,
        webhook_port=18081,
        telegram_api_port=18080,
        valkey_port=6379,
        valkey_url="redis://127.0.0.1:6379/0",
        valkey_prefix="omni-agent:session:ci:test",
        username="ci-user",
        webhook_secret="test-webhook-secret",
        chat_id=1001,
        chat_b=1002,
        chat_c=1003,
        user_id=2001,
        user_b=2002,
        user_c=2003,
        runtime_log_file=tmp_path / "runtime.log",
        mock_log_file=tmp_path / "mock.log",
        runtime_startup_timeout_secs=90,
        quick_max_wait=45,
        quick_max_idle=25,
        full_max_wait=90,
        full_max_idle=40,
        matrix_max_wait=45,
        matrix_max_idle=30,
        benchmark_iterations=3,
        skip_matrix=False,
        skip_benchmark=False,
        skip_evolution=False,
        skip_rust_regressions=False,
        skip_discover_cache_gate=False,
        skip_reflection_quality_gate=False,
        skip_trace_reconstruction_gate=False,
        skip_cross_group_complex_gate=False,
        evolution_report_json=tmp_path / "evolution.json",
        benchmark_report_json=tmp_path / "benchmark.json",
        session_matrix_report_json=tmp_path / "session-matrix.json",
        session_matrix_report_markdown=tmp_path / "session-matrix.md",
        trace_report_json=tmp_path / "trace-reconstruction.json",
        trace_report_markdown=tmp_path / "trace-reconstruction.md",
        cross_group_report_json=tmp_path / "cross-group-complex.json",
        cross_group_report_markdown=tmp_path / "cross-group-complex.md",
        cross_group_dataset=tmp_path / "complex-dataset.json",
        cross_group_scenario_id="cross_group_control_plane_stress",
        min_planned_hits=10,
        min_successful_corrections=3,
        min_recall_credit_events=1,
        min_quality_score=90.0,
        slow_response_min_duration_ms=20000,
        slow_response_long_step_ms=1200,
        slow_response_min_long_steps=1,
        trace_min_quality_score=90.0,
        trace_max_events=2000,
        min_session_steps=20,
        require_cross_group_step=True,
        require_mixed_batch_steps=True,
        cross_group_max_wait=90,
        cross_group_max_idle=80,
        cross_group_max_parallel=3,
        discover_cache_hit_p95_ms=15.0,
        discover_cache_miss_p95_ms=80.0,
        discover_cache_bench_iterations=12,
        max_mcp_call_waiting_events=0,
        max_mcp_connect_waiting_events=0,
        max_mcp_waiting_events_total=0,
        max_memory_stream_read_failed_events=0,
        max_embedding_timeout_fallback_turns=0,
        max_embedding_cooldown_fallback_turns=0,
        max_embedding_unavailable_fallback_turns=0,
        max_embedding_fallback_turns_total=0,
    )


def write_report(cfg: GateConfig, payload: dict[str, object]) -> None:
    """Write JSON report at session matrix report path."""
    cfg.session_matrix_report_json.parent.mkdir(parents=True, exist_ok=True)
    cfg.session_matrix_report_json.write_text(
        json.dumps(payload, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )


def passing_report(cfg: GateConfig) -> dict[str, object]:
    """Build passing session matrix report payload."""
    steps = [{"name": f"step-{index}", "passed": True} for index in range(1, 18)]
    steps.extend(
        [
            {"name": "concurrent_cross_group", "passed": True},
            {"name": "mixed_reset_session_a", "passed": True},
            {"name": "mixed_resume_status_session_b", "passed": True},
            {"name": "mixed_plain_session_c", "passed": True},
        ]
    )
    return {
        "overall_passed": True,
        "summary": {"total": len(steps), "failed": 0},
        "config": {
            "chat_id": cfg.chat_id,
            "chat_b": cfg.chat_b,
            "chat_c": cfg.chat_c,
        },
        "steps": steps,
    }
