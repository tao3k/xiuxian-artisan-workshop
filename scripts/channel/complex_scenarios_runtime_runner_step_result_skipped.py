#!/usr/bin/env python3
"""Skipped-step result builder for complex scenario runner."""

from __future__ import annotations

from typing import Any


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
