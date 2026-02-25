#!/usr/bin/env python3
"""Gate-stage execution bindings for memory CI runner."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def run_trace_reconstruction_gate(
    cfg: Any,
    *,
    cwd: Path,
    env: dict[str, str],
    runtime_module: Any,
    run_command_fn: Any,
    assert_trace_reconstruction_quality_fn: Any,
) -> None:
    """Run trace reconstruction gate stage."""
    runtime_module.run_trace_reconstruction_gate(
        cfg,
        cwd=cwd,
        env=env,
        run_command_fn=run_command_fn,
        assert_trace_reconstruction_quality_fn=assert_trace_reconstruction_quality_fn,
    )


def run_cross_group_complex_gate(
    cfg: Any,
    *,
    cwd: Path,
    env: dict[str, str],
    runtime_module: Any,
    run_command_fn: Any,
    assert_cross_group_complex_quality_fn: Any,
) -> None:
    """Run cross-group complex gate stage."""
    runtime_module.run_cross_group_complex_gate(
        cfg,
        cwd=cwd,
        env=env,
        run_command_fn=run_command_fn,
        assert_cross_group_complex_quality_fn=assert_cross_group_complex_quality_fn,
    )


def run_gate(
    cfg: Any,
    *,
    runner_module: Any,
    ensure_parent_dirs_fn: Any,
    default_run_suffix_fn: Any,
    write_ci_channel_acl_settings_fn: Any,
    valkey_reachable_fn: Any,
    run_command_fn: Any,
    start_background_process_fn: Any,
    wait_for_mock_health_fn: Any,
    wait_for_log_regex_fn: Any,
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
    terminate_process_fn: Any,
) -> None:
    """Run full CI gate workflow."""
    runner_module.run_gate(
        cfg,
        ensure_parent_dirs_fn=ensure_parent_dirs_fn,
        default_run_suffix_fn=default_run_suffix_fn,
        write_ci_channel_acl_settings_fn=write_ci_channel_acl_settings_fn,
        valkey_reachable_fn=valkey_reachable_fn,
        run_command_fn=run_command_fn,
        start_background_process_fn=start_background_process_fn,
        wait_for_mock_health_fn=wait_for_mock_health_fn,
        wait_for_log_regex_fn=wait_for_log_regex_fn,
        run_reflection_quality_gate_fn=run_reflection_quality_gate_fn,
        run_discover_cache_gate_fn=run_discover_cache_gate_fn,
        run_trace_reconstruction_gate_fn=run_trace_reconstruction_gate_fn,
        run_cross_group_complex_gate_fn=run_cross_group_complex_gate_fn,
        assert_mcp_waiting_warning_budget_fn=assert_mcp_waiting_warning_budget_fn,
        assert_memory_stream_warning_budget_fn=assert_memory_stream_warning_budget_fn,
        assert_evolution_quality_fn=assert_evolution_quality_fn,
        assert_evolution_slow_response_quality_fn=assert_evolution_slow_response_quality_fn,
        assert_session_matrix_quality_fn=assert_session_matrix_quality_fn,
        assert_benchmark_quality_fn=assert_benchmark_quality_fn,
        terminate_process_fn=terminate_process_fn,
    )
