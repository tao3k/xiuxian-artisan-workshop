#!/usr/bin/env python3
"""Payload building helpers for command event probe reports."""

from __future__ import annotations

import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


def _build_summary(attempts: list[Any]) -> dict[str, int]:
    """Compute aggregate pass/fail metrics for probe attempts."""
    passed = sum(1 for attempt in attempts if attempt.passed)
    failed = len(attempts) - passed
    retry_scheduled = sum(1 for attempt in attempts if attempt.retry_scheduled)
    return {
        "total": len(attempts),
        "passed": passed,
        "failed": failed,
        "retry_scheduled": retry_scheduled,
    }


def _build_config(
    *,
    suites: tuple[str, ...],
    case_ids: tuple[str, ...],
    runtime_partition_mode: str | None,
    admin_matrix: bool,
    assert_admin_isolation: bool,
    assert_admin_topic_isolation: bool,
    group_thread_id: int | None,
    group_thread_id_b: int | None,
    matrix_chat_ids: tuple[int, ...],
    allow_chat_ids: tuple[str, ...],
    max_wait: int,
    max_idle_secs: int,
    matrix_retries: int,
    matrix_backoff_secs: float,
) -> dict[str, object]:
    """Build immutable report configuration snapshot."""
    return {
        "suites": list(suites),
        "cases": list(case_ids),
        "runtime_partition_mode": runtime_partition_mode,
        "admin_matrix": bool(admin_matrix),
        "assert_admin_isolation": bool(assert_admin_isolation),
        "assert_admin_topic_isolation": bool(assert_admin_topic_isolation),
        "group_thread_id": group_thread_id,
        "group_thread_id_b": group_thread_id_b,
        "matrix_chat_ids": list(matrix_chat_ids),
        "allow_chat_ids": list(allow_chat_ids),
        "max_wait": int(max_wait),
        "max_idle_secs": int(max_idle_secs),
        "matrix_retries": int(matrix_retries),
        "matrix_backoff_secs": float(matrix_backoff_secs),
    }


def build_report(
    *,
    suites: tuple[str, ...],
    case_ids: tuple[str, ...],
    allow_chat_ids: tuple[str, ...],
    matrix_chat_ids: tuple[int, ...],
    attempts: list[Any],
    started_dt: datetime,
    started_mono: float,
    exit_code: int,
    runtime_partition_mode: str | None,
    admin_matrix: bool,
    assert_admin_isolation: bool,
    assert_admin_topic_isolation: bool,
    group_thread_id: int | None,
    group_thread_id_b: int | None,
    max_wait: int,
    max_idle_secs: int,
    matrix_retries: int,
    matrix_backoff_secs: float,
) -> dict[str, object]:
    """Build structured report payload from probe attempts and run config."""
    finished_dt = datetime.now(UTC)
    duration_ms = int((time.monotonic() - started_mono) * 1000)
    summary = _build_summary(attempts)
    return {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": duration_ms,
        "exit_code": exit_code,
        "overall_passed": exit_code == 0 and summary["failed"] == 0 and len(attempts) > 0,
        "summary": summary,
        "config": _build_config(
            suites=suites,
            case_ids=case_ids,
            runtime_partition_mode=runtime_partition_mode,
            admin_matrix=admin_matrix,
            assert_admin_isolation=assert_admin_isolation,
            assert_admin_topic_isolation=assert_admin_topic_isolation,
            group_thread_id=group_thread_id,
            group_thread_id_b=group_thread_id_b,
            matrix_chat_ids=matrix_chat_ids,
            allow_chat_ids=allow_chat_ids,
            max_wait=max_wait,
            max_idle_secs=max_idle_secs,
            matrix_retries=matrix_retries,
            matrix_backoff_secs=matrix_backoff_secs,
        ),
        "attempts": [asdict(attempt) for attempt in attempts],
    }
