#!/usr/bin/env python3
"""Session-matrix quality gate assertions for memory CI."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_quality_common import load_json


def assert_session_matrix_quality(cfg: Any) -> None:
    """Validate session matrix coverage and expected step behavior."""
    report = load_json(cfg.session_matrix_report_json)
    if not bool(report.get("overall_passed", False)):
        raise RuntimeError("session matrix report indicates overall failure")

    summary_obj = report.get("summary")
    summary = summary_obj if isinstance(summary_obj, dict) else {}
    steps_total = int(summary.get("total", 0))
    steps_failed = int(summary.get("failed", 0))

    steps_obj = report.get("steps")
    steps = steps_obj if isinstance(steps_obj, list) else []
    steps_by_name: dict[str, dict[str, object]] = {}
    for item in steps:
        if not isinstance(item, dict):
            continue
        name = str(item.get("name", "")).strip()
        if not name:
            continue
        steps_by_name[name] = item

    expected_min_session_steps = cfg.min_session_steps
    if (
        "concurrent_baseline_cross_chat" in steps_by_name
        and "concurrent_cross_group" not in steps_by_name
    ):
        expected_min_session_steps = max(1, cfg.min_session_steps - 1)

    if steps_total < expected_min_session_steps:
        raise RuntimeError(
            "session matrix steps below threshold: "
            f"total={steps_total} < min_session_steps={expected_min_session_steps}"
        )
    if steps_failed > 0:
        raise RuntimeError(f"session matrix has failed steps: failed={steps_failed}")

    config_obj = report.get("config")
    config = config_obj if isinstance(config_obj, dict) else {}
    chat_ids = (
        int(config.get("chat_id", cfg.chat_id)),
        int(config.get("chat_b", cfg.chat_b)),
        int(config.get("chat_c", cfg.chat_c)),
    )
    if len(set(chat_ids)) < 3:
        raise RuntimeError(
            f"session matrix did not run with three distinct groups: chat_ids={chat_ids}"
        )

    if cfg.require_cross_group_step:
        cross_group = steps_by_name.get("concurrent_cross_group")
        cross_chat_baseline = steps_by_name.get("concurrent_baseline_cross_chat")
        if not isinstance(cross_group, dict) and not isinstance(cross_chat_baseline, dict):
            raise RuntimeError("session matrix missing required step: concurrent_cross_group")
        if isinstance(cross_group, dict) and not bool(cross_group.get("passed", False)):
            raise RuntimeError("session matrix cross-group step failed")
        if isinstance(cross_chat_baseline, dict) and not bool(
            cross_chat_baseline.get("passed", False)
        ):
            raise RuntimeError("session matrix cross-group baseline step failed")

    if cfg.require_mixed_batch_steps:
        required_mixed = (
            "mixed_reset_session_a",
            "mixed_resume_status_session_b",
            "mixed_plain_session_c",
        )
        missing = [name for name in required_mixed if name not in steps_by_name]
        if missing:
            raise RuntimeError(f"session matrix missing mixed batch steps: {missing}")
        failed = [
            name for name in required_mixed if not bool(steps_by_name[name].get("passed", False))
        ]
        if failed:
            raise RuntimeError(f"session matrix mixed batch steps failed: {failed}")

    print(
        "Session matrix quality gates passed: "
        f"steps_total={steps_total}, cross_group={'on' if cfg.require_cross_group_step else 'off'}, "
        f"mixed_batch={'on' if cfg.require_mixed_batch_steps else 'off'}",
        flush=True,
    )
