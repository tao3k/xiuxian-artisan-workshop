#!/usr/bin/env python3
"""Execution bindings for complex-scenarios runtime entrypoint."""

from __future__ import annotations

from typing import Any


def tail_text(value: str, limit_lines: int = 40) -> str:
    """Return bounded tail text for probe stdout/stderr."""
    lines = value.splitlines()
    if len(lines) <= limit_lines:
        return value
    return "\n".join(lines[-limit_lines:])


def run_step(
    cfg: Any,
    scenario_id: str,
    step: Any,
    session: Any,
    wave_index: int,
    *,
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
    """Run one scenario step."""
    return execution_module.run_step(
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
    execution_module: Any,
    runtime_partition_mode: str | None = None,
    expected_session_key_fn: Any,
    step_run_result_cls: Any,
) -> Any:
    """Build skipped step result."""
    return execution_module.skipped_step_result(
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
    """Run one full scenario with complexity + quality evaluation."""
    return execution_module.run_scenario(
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
