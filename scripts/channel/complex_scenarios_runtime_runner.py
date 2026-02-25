#!/usr/bin/env python3
"""Compatibility wrappers for complex scenario runtime runner helpers."""

from __future__ import annotations

from typing import Any

from complex_scenarios_runtime_runner_flow import run_scenario as _run_scenario_flow
from complex_scenarios_runtime_runner_step import (
    run_step as _run_step_impl,
)
from complex_scenarios_runtime_runner_step import (
    skipped_step_result as _skipped_step_result_impl,
)


def run_step(
    cfg: Any,
    scenario_id: str,
    step: Any,
    session: Any,
    wave_index: int,
    *,
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
    """Execute one scenario step and build a typed result object."""
    return _run_step_impl(
        cfg,
        scenario_id,
        step,
        session,
        wave_index,
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
    runtime_partition_mode: str | None,
    expected_session_key_fn: Any,
    step_run_result_cls: Any,
) -> Any:
    """Build a skipped step result (dependency blocked / unreachable)."""
    return _skipped_step_result_impl(
        scenario_id,
        step,
        session,
        wave_index,
        reason,
        runtime_partition_mode=runtime_partition_mode,
        expected_session_key_fn=expected_session_key_fn,
        step_run_result_cls=step_run_result_cls,
    )


def run_scenario(
    cfg: Any,
    scenario: Any,
    *,
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
    """Execute all scenario waves, then evaluate complexity + quality."""
    return _run_scenario_flow(
        cfg,
        scenario,
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
