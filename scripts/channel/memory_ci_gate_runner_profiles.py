#!/usr/bin/env python3
"""Profile-specific execution flows for memory CI gate runner."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_runner_profiles_commands import (
    build_benchmark_cmd,
    build_nightly_suite_cmd,
    build_quick_suite_cmd,
    build_session_matrix_cmd,
)
from memory_ci_gate_runner_profiles_gates import run_common_post_gates


def run_quick_profile(
    cfg: Any,
    *,
    env: dict[str, str],
    memory_suite: Any,
    run_command_fn: Any,
    run_reflection_quality_gate_fn: Any,
    run_discover_cache_gate_fn: Any,
    run_trace_reconstruction_gate_fn: Any,
    assert_mcp_waiting_warning_budget_fn: Any,
    assert_memory_stream_warning_budget_fn: Any,
) -> None:
    """Run quick profile CI gate workflow."""
    quick_suite_cmd = build_quick_suite_cmd(cfg, memory_suite)
    run_command_fn(
        quick_suite_cmd,
        title="Quick gate: memory suite (black-box + Rust regressions, evolution skipped)",
        cwd=cfg.project_root,
        env=env,
    )
    run_common_post_gates(
        cfg,
        env=env,
        run_reflection_quality_gate_fn=run_reflection_quality_gate_fn,
        run_discover_cache_gate_fn=run_discover_cache_gate_fn,
        run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate_fn,
        assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget_fn,
        assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget_fn,
    )


def run_nightly_profile(
    cfg: Any,
    *,
    env: dict[str, str],
    memory_suite: Any,
    session_matrix: Any,
    memory_benchmark: Any,
    run_command_fn: Any,
    assert_evolution_quality_fn: Any,
    assert_evolution_slow_response_quality_fn: Any,
    assert_session_matrix_quality_fn: Any,
    assert_benchmark_quality_fn: Any,
    run_cross_group_complex_gate_fn: Any,
    run_reflection_quality_gate_fn: Any,
    run_discover_cache_gate_fn: Any,
    run_trace_reconstruction_gate_fn: Any,
    assert_mcp_waiting_warning_budget_fn: Any,
    assert_memory_stream_warning_budget_fn: Any,
) -> None:
    """Run nightly profile CI gate workflow."""
    nightly_suite_cmd = build_nightly_suite_cmd(cfg, memory_suite)
    run_command_fn(
        nightly_suite_cmd,
        title="Nightly gate: full memory suite (includes evolution DAG + Rust regressions)",
        cwd=cfg.project_root,
        env=env,
    )
    if not cfg.skip_evolution:
        assert_evolution_quality_fn(cfg)
        assert_evolution_slow_response_quality_fn(cfg)
    else:
        print("Skipping slow-response resilience gate because evolution is skipped.", flush=True)

    if not cfg.skip_matrix:
        run_command_fn(
            build_session_matrix_cmd(cfg, session_matrix),
            title="Nightly gate: session matrix",
            cwd=cfg.project_root,
            env=env,
        )
        assert_session_matrix_quality_fn(cfg)

    run_cross_group_complex_gate_fn(cfg, cwd=cfg.project_root, env=env)

    if not cfg.skip_benchmark:
        run_command_fn(
            build_benchmark_cmd(cfg, memory_benchmark),
            title="Nightly gate: memory A/B benchmark",
            cwd=cfg.project_root,
            env=env,
        )
        assert_benchmark_quality_fn(cfg)

    run_common_post_gates(
        cfg,
        env=env,
        run_reflection_quality_gate_fn=run_reflection_quality_gate_fn,
        run_discover_cache_gate_fn=run_discover_cache_gate_fn,
        run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate_fn,
        assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget_fn,
        assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget_fn,
    )
