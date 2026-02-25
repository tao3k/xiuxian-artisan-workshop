#!/usr/bin/env python3
"""Runtime execution bindings for complex scenarios runner."""

from __future__ import annotations

from typing import Any


def run_step(
    cfg: Any,
    scenario_id: str,
    step: Any,
    session: Any,
    wave_index: int,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    expected_session_key_fn: Any,
    expected_session_log_regex_fn: Any,
    run_cmd_fn: Any,
    detect_memory_event_flags_fn: Any,
    extract_memory_metrics_fn: Any,
    extract_mcp_metrics_fn: Any,
    extract_bot_excerpt_fn: Any,
    tail_text_fn: Any,
    step_run_result_cls: Any,
) -> Any:
    """Run one step through runtime bindings."""
    return runtime_bindings_module.run_step(
        cfg,
        scenario_id,
        step,
        session,
        wave_index,
        execution_module=execution_module,
        expected_session_key_fn=expected_session_key_fn,
        expected_session_log_regex_fn=expected_session_log_regex_fn,
        run_cmd_fn=run_cmd_fn,
        detect_memory_event_flags_fn=detect_memory_event_flags_fn,
        extract_memory_metrics_fn=extract_memory_metrics_fn,
        extract_mcp_metrics_fn=extract_mcp_metrics_fn,
        extract_bot_excerpt_fn=extract_bot_excerpt_fn,
        tail_text_fn=tail_text_fn,
        step_run_result_cls=step_run_result_cls,
    )


def skipped_step_result(
    scenario_id: str,
    step: Any,
    session: Any,
    wave_index: int,
    reason: str,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    runtime_partition_mode: str | None,
    expected_session_key_fn: Any,
    step_run_result_cls: Any,
) -> Any:
    """Build skipped-step result through runtime bindings."""
    return runtime_bindings_module.skipped_step_result(
        scenario_id,
        step,
        session,
        wave_index,
        reason,
        execution_module=execution_module,
        runtime_partition_mode=runtime_partition_mode,
        expected_session_key_fn=expected_session_key_fn,
        step_run_result_cls=step_run_result_cls,
    )


def run_scenario(
    cfg: Any,
    scenario: Any,
    *,
    runtime_bindings_module: Any,
    execution_module: Any,
    merge_requirements_fn: Any,
    merge_quality_requirements_fn: Any,
    compute_complexity_profile_fn: Any,
    evaluate_complexity_fn: Any,
    quality_profile_cls: Any,
    build_execution_waves_fn: Any,
    run_step_fn: Any,
    skipped_step_result_fn: Any,
    compute_quality_profile_fn: Any,
    evaluate_quality_fn: Any,
    scenario_run_result_cls: Any,
) -> Any:
    """Run one scenario through runtime bindings."""
    return runtime_bindings_module.run_scenario(
        cfg,
        scenario,
        execution_module=execution_module,
        merge_requirements_fn=merge_requirements_fn,
        merge_quality_requirements_fn=merge_quality_requirements_fn,
        compute_complexity_profile_fn=compute_complexity_profile_fn,
        evaluate_complexity_fn=evaluate_complexity_fn,
        quality_profile_cls=quality_profile_cls,
        build_execution_waves_fn=build_execution_waves_fn,
        run_step_fn=run_step_fn,
        skipped_step_result_fn=skipped_step_result_fn,
        compute_quality_profile_fn=compute_quality_profile_fn,
        evaluate_quality_fn=evaluate_quality_fn,
        scenario_run_result_cls=scenario_run_result_cls,
    )
