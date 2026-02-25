#!/usr/bin/env python3
"""Step-level execution helpers for complex runtime scenarios."""

from __future__ import annotations

from typing import Any

from complex_scenarios_runtime_runner_step_command import build_step_command
from complex_scenarios_runtime_runner_step_result import (
    build_skipped_step_result,
    build_step_result,
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
    session_key = expected_session_key_fn(
        session.chat_id,
        session.user_id,
        session.thread_id,
        cfg.runtime_partition_mode,
    )
    cmd, _ = build_step_command(
        cfg=cfg,
        step=step,
        session=session,
        expected_session_log_regex_fn=expected_session_log_regex_fn,
    )

    returncode, duration_ms, stdout, stderr = run_cmd_fn(cmd)
    return build_step_result(
        scenario_id=scenario_id,
        step=step,
        session_key=session_key,
        wave_index=wave_index,
        cmd=cmd,
        returncode=returncode,
        duration_ms=duration_ms,
        stdout=stdout,
        stderr=stderr,
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
    return build_skipped_step_result(
        scenario_id=scenario_id,
        session_key=expected_session_key_fn(
            session.chat_id,
            session.user_id,
            session.thread_id,
            runtime_partition_mode,
        ),
        step=step,
        wave_index=wave_index,
        reason=reason,
        step_run_result_cls=step_run_result_cls,
    )
