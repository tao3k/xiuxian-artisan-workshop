#!/usr/bin/env python3
"""Profile execution helpers for memory CI gate workflow."""

from __future__ import annotations

from typing import Any


def run_profile(
    cfg: Any,
    *,
    env: dict[str, str],
    script_paths: dict[str, Any],
    profiles_module: Any,
    run_command_fn: Any,
    run_reflection_quality_gate_fn: Any,
    run_discover_cache_gate_fn: Any,
    run_trace_reconstruction_gate_fn: Any,
    run_cross_group_complex_gate_fn: Any,
    assert_mcp_waiting_warning_budget_fn: Any,
    assert_memory_stream_warning_budget_fn: Any,
    assert_evolution_quality_fn: Any,
    assert_evolution_slow_response_quality_fn: Any,
    assert_session_matrix_quality_fn: Any,
    assert_benchmark_quality_fn: Any,
) -> None:
    """Run quick/nightly gate profile through profile module hooks."""
    memory_suite = script_paths["memory_suite"]
    session_matrix = script_paths["session_matrix"]
    memory_benchmark = script_paths["memory_benchmark"]

    if cfg.profile == "quick":
        profiles_module.run_quick_profile(
            cfg,
            env=env,
            memory_suite=memory_suite,
            run_command_fn=run_command_fn,
            run_reflection_quality_gate_fn=run_reflection_quality_gate_fn,
            run_discover_cache_gate_fn=run_discover_cache_gate_fn,
            run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate_fn,
            assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget_fn,
            assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget_fn,
        )
        return

    profiles_module.run_nightly_profile(
        cfg,
        env=env,
        memory_suite=memory_suite,
        session_matrix=session_matrix,
        memory_benchmark=memory_benchmark,
        run_command_fn=run_command_fn,
        assert_evolution_quality_fn=assert_evolution_quality_fn,
        assert_evolution_slow_response_quality_fn=assert_evolution_slow_response_quality_fn,
        assert_session_matrix_quality_fn=assert_session_matrix_quality_fn,
        assert_benchmark_quality_fn=assert_benchmark_quality_fn,
        run_cross_group_complex_gate_fn=run_cross_group_complex_gate_fn,
        run_reflection_quality_gate_fn=run_reflection_quality_gate_fn,
        run_discover_cache_gate_fn=run_discover_cache_gate_fn,
        run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate_fn,
        assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget_fn,
        assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget_fn,
    )
