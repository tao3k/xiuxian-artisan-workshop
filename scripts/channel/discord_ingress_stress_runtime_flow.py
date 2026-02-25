#!/usr/bin/env python3
"""Execution flow helpers for Discord ingress stress runtime."""

from __future__ import annotations

import importlib
import time
from typing import Any

_rounds_module = importlib.import_module("discord_ingress_stress_runtime_rounds")
_summary_module = importlib.import_module("discord_ingress_stress_runtime_summary")


def run_worker_dynamic(
    cfg: Any,
    round_index: int,
    worker_index: int,
    *,
    next_event_id_fn: Any,
    build_ingress_payload_fn: Any,
    post_ingress_event_fn: Any,
) -> dict[str, Any]:
    """Run one worker by delegating to rounds helper module."""
    return _rounds_module.run_worker(
        cfg,
        round_index,
        worker_index,
        next_event_id_fn=next_event_id_fn,
        build_ingress_payload_fn=build_ingress_payload_fn,
        post_ingress_event_fn=post_ingress_event_fn,
    )


def run_stress(
    cfg: Any,
    *,
    round_result_cls: Any,
    utc_now_fn: Any,
    p95_fn: Any,
    init_log_offset_fn: Any,
    read_new_log_lines_fn: Any,
    run_worker_fn: Any,
) -> dict[str, object]:
    """Execute full Discord ingress stress run and return structured report."""
    started_at = utc_now_fn()
    run_started = time.perf_counter()

    rounds: list[Any] = []
    total_rounds = cfg.warmup_rounds + cfg.rounds
    for index in range(total_rounds):
        warmup = index < cfg.warmup_rounds
        round_index = index + 1
        row = _rounds_module.run_round(
            cfg,
            round_index,
            warmup=warmup,
            round_result_cls=round_result_cls,
            init_log_offset_fn=init_log_offset_fn,
            read_new_log_lines_fn=read_new_log_lines_fn,
            run_worker_fn=run_worker_fn,
            p95_fn=p95_fn,
        )
        rounds.append(row)

        print(
            "[round {}/{}] warmup={} req={} ok={} fail={} p95_ms={:.1f} rps={:.1f}".format(
                round_index,
                total_rounds,
                "yes" if warmup else "no",
                row.total_requests,
                row.success_requests,
                row.failed_requests,
                row.p95_latency_ms,
                row.rps,
            )
        )

        if index + 1 < total_rounds and cfg.cooldown_secs > 0:
            time.sleep(cfg.cooldown_secs)

    measured = [row for row in rounds if not row.warmup]
    quality_passed, quality_failures = _summary_module.evaluate_quality(cfg, measured)

    finished_at = utc_now_fn()
    duration_ms = int((time.perf_counter() - run_started) * 1000)
    return _summary_module.build_report(
        cfg,
        started_at=started_at,
        finished_at=finished_at,
        duration_ms=duration_ms,
        rounds=rounds,
        measured=measured,
        quality_passed=quality_passed,
        quality_failures=quality_failures,
    )
