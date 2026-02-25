#!/usr/bin/env python3
"""Result construction helpers for complex scenario step runs."""

from __future__ import annotations

from typing import Any


def build_step_result(
    *,
    scenario_id: str,
    step: Any,
    session_key: str,
    wave_index: int,
    cmd: list[str],
    returncode: int,
    duration_ms: int,
    stdout: str,
    stderr: str,
    detect_memory_event_flags_fn: Any,
    extract_memory_metrics_fn: Any,
    extract_mcp_metrics_fn: Any,
    extract_bot_excerpt_fn: Any,
    tail_text_fn: Any,
    step_run_result_cls: Any,
) -> Any:
    """Build successful/failed (non-skipped) step result payload."""
    passed = returncode == 0
    memory_planned_seen, memory_injected_seen, memory_skipped_seen, memory_feedback_updated_seen = (
        detect_memory_event_flags_fn(stdout)
    )
    memory_metrics = extract_memory_metrics_fn(stdout)
    mcp_metrics = extract_mcp_metrics_fn(stdout)
    recall_credit_count = int(memory_metrics.get("memory_recall_credit_count") or 0)
    decay_count = int(memory_metrics.get("memory_decay_count") or 0)
    return step_run_result_cls(
        scenario_id=scenario_id,
        step_id=step.step_id,
        session_alias=step.session_alias,
        session_key=session_key,
        wave_index=wave_index,
        depends_on=step.depends_on,
        prompt=step.prompt,
        event=step.expect_event,
        command=tuple(cmd),
        returncode=returncode,
        duration_ms=duration_ms,
        passed=passed,
        skipped=False,
        skip_reason=None,
        bot_excerpt=extract_bot_excerpt_fn(stdout),
        memory_planned_seen=memory_planned_seen,
        memory_injected_seen=memory_injected_seen,
        memory_skipped_seen=memory_skipped_seen,
        memory_feedback_updated_seen=memory_feedback_updated_seen,
        memory_recall_credit_seen=recall_credit_count > 0,
        memory_decay_seen=decay_count > 0,
        memory_recall_credit_count=recall_credit_count,
        memory_decay_count=decay_count,
        memory_planned_bias=memory_metrics["memory_planned_bias"],  # type: ignore[arg-type]
        memory_decision=memory_metrics["memory_decision"],  # type: ignore[arg-type]
        mcp_last_event=mcp_metrics["mcp_last_event"],  # type: ignore[arg-type]
        mcp_waiting_seen=bool(mcp_metrics["mcp_waiting_seen"]),
        mcp_event_counts=dict(mcp_metrics["mcp_event_counts"]),  # type: ignore[arg-type]
        feedback_command_bias_before=memory_metrics["feedback_command_bias_before"],  # type: ignore[arg-type]
        feedback_command_bias_after=memory_metrics["feedback_command_bias_after"],  # type: ignore[arg-type]
        feedback_command_bias_delta=memory_metrics["feedback_command_bias_delta"],  # type: ignore[arg-type]
        feedback_heuristic_bias_before=memory_metrics["feedback_heuristic_bias_before"],  # type: ignore[arg-type]
        feedback_heuristic_bias_after=memory_metrics["feedback_heuristic_bias_after"],  # type: ignore[arg-type]
        feedback_heuristic_bias_delta=memory_metrics["feedback_heuristic_bias_delta"],  # type: ignore[arg-type]
        stdout_tail=tail_text_fn(stdout),
        stderr_tail=tail_text_fn(stderr),
    )


def build_skipped_step_result(
    *,
    scenario_id: str,
    step: Any,
    session_key: str,
    wave_index: int,
    reason: str,
    step_run_result_cls: Any,
) -> Any:
    """Build skipped step result (dependency blocked / unreachable)."""
    return step_run_result_cls(
        scenario_id=scenario_id,
        step_id=step.step_id,
        session_alias=step.session_alias,
        session_key=session_key,
        wave_index=wave_index,
        depends_on=step.depends_on,
        prompt=step.prompt,
        event=step.expect_event,
        command=(),
        returncode=1,
        duration_ms=0,
        passed=False,
        skipped=True,
        skip_reason=reason,
        bot_excerpt=None,
        memory_planned_seen=False,
        memory_injected_seen=False,
        memory_skipped_seen=False,
        memory_feedback_updated_seen=False,
        memory_recall_credit_seen=False,
        memory_decay_seen=False,
        memory_recall_credit_count=0,
        memory_decay_count=0,
        memory_planned_bias=None,
        memory_decision=None,
        mcp_last_event=None,
        mcp_waiting_seen=False,
        mcp_event_counts={},
        feedback_command_bias_before=None,
        feedback_command_bias_after=None,
        feedback_command_bias_delta=None,
        feedback_heuristic_bias_before=None,
        feedback_heuristic_bias_after=None,
        feedback_heuristic_bias_delta=None,
        stdout_tail="",
        stderr_tail="",
    )
