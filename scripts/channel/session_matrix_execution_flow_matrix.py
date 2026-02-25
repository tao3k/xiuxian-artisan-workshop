#!/usr/bin/env python3
"""Matrix-level orchestration for session-matrix execution flow."""

from __future__ import annotations

import time
from datetime import UTC, datetime
from typing import Any


def run_matrix(
    cfg: Any,
    *,
    script_dir: Any,
    build_report_fn: Any,
    build_matrix_steps_fn: Any,
    run_concurrent_step_fn: Any,
    run_blackbox_step_fn: Any,
    run_mixed_concurrency_batch_fn: Any,
) -> tuple[bool, dict[str, object]]:
    """Run the full session matrix and return overall status + report."""
    started_dt = datetime.now(UTC)
    started_mono = time.monotonic()
    results: list[Any] = []

    baseline_name = "concurrent_baseline_same_chat"
    baseline_chat_b = cfg.chat_id
    baseline_user_b = cfg.user_b
    baseline_thread_b = cfg.thread_b
    baseline_allow_send_failure = False
    baseline_is_cross_group = False

    if cfg.session_partition == "chat":
        baseline_name = "concurrent_baseline_cross_chat"
        if cfg.chat_b != cfg.chat_id:
            baseline_chat_b = cfg.chat_b
            baseline_user_b = cfg.user_b
            baseline_thread_b = cfg.thread_b
        else:
            baseline_chat_b = cfg.chat_c
            baseline_user_b = cfg.user_c
            baseline_thread_b = cfg.thread_c
        baseline_is_cross_group = True

    results.append(
        run_concurrent_step_fn(
            script_dir,
            cfg,
            name=baseline_name,
            chat_a=cfg.chat_id,
            user_a=cfg.user_a,
            thread_a=cfg.thread_a,
            chat_b=baseline_chat_b,
            user_b=baseline_user_b,
            thread_b=baseline_thread_b,
            prompt="/session json",
            allow_send_failure=baseline_allow_send_failure,
        )
    )
    if not results[-1].passed:
        return False, build_report_fn(cfg, results, started_dt, started_mono)

    if cfg.chat_b != cfg.chat_id and not baseline_is_cross_group:
        results.append(
            run_concurrent_step_fn(
                script_dir,
                cfg,
                name="concurrent_cross_group",
                chat_a=cfg.chat_id,
                user_a=cfg.user_a,
                thread_a=cfg.thread_a,
                chat_b=cfg.chat_b,
                user_b=cfg.user_b,
                thread_b=cfg.thread_b,
                prompt="/session json",
                allow_send_failure=True,
            )
        )
        if not results[-1].passed:
            return False, build_report_fn(cfg, results, started_dt, started_mono)

    if (
        cfg.session_partition == "chat_thread_user"
        and cfg.thread_a is not None
        and cfg.thread_b is not None
        and cfg.thread_a != cfg.thread_b
    ):
        results.append(
            run_concurrent_step_fn(
                script_dir,
                cfg,
                name="concurrent_cross_thread_same_user",
                chat_a=cfg.chat_id,
                user_a=cfg.user_a,
                thread_a=cfg.thread_a,
                chat_b=cfg.chat_id,
                user_b=cfg.user_a,
                thread_b=cfg.thread_b,
                prompt="/session json",
                allow_send_failure=False,
            )
        )
        if not results[-1].passed:
            return False, build_report_fn(cfg, results, started_dt, started_mono)

    for step in build_matrix_steps_fn(cfg):
        result = run_blackbox_step_fn(script_dir, cfg, step)
        results.append(result)
        if not result.passed:
            break

    if all(step.passed for step in results):
        for mixed_result in run_mixed_concurrency_batch_fn(script_dir, cfg):
            results.append(mixed_result)
            if not mixed_result.passed:
                break

    overall_passed = all(step.passed for step in results)
    return overall_passed, build_report_fn(cfg, results, started_dt, started_mono)
