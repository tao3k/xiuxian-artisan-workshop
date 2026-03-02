#!/usr/bin/env python3
"""Execution helpers for MCP startup stress flow."""

from __future__ import annotations

import concurrent.futures
import threading
import time
from datetime import UTC, datetime
from typing import Any

from mcp_startup_stress_flow_run_health import (
    prepare_health_preflight,
    start_health_sampler,
    stop_health_sampler,
)
from mcp_startup_stress_flow_run_report import build_run_report


def run_stress(
    cfg: Any,
    *,
    probe_result_cls: Any,
    health_sample_cls: Any,
    check_health_fn: Any,
    run_restart_command_fn: Any,
    collect_health_sample_fn: Any,
    run_single_probe_fn: Any,
    summarize_fn: Any,
) -> dict[str, object]:
    """Execute full stress run including optional restart and health sampling."""
    started_dt = datetime.now(UTC)
    started = time.monotonic()
    health_preflight = prepare_health_preflight(cfg, check_health_fn=check_health_fn)

    results: list[Any] = []
    health_samples: list[Any] = []
    health_samples_lock = threading.Lock()
    health_stop = threading.Event()
    restart_events: list[dict[str, object]] = []

    health_thread = start_health_sampler(
        cfg,
        health_sample_cls=health_sample_cls,
        collect_health_sample_fn=collect_health_sample_fn,
        health_samples=health_samples,
        health_samples_lock=health_samples_lock,
        health_stop=health_stop,
    )

    try:
        for round_index in range(1, cfg.rounds + 1):
            if round_index > 1 and cfg.restart_mcp_cmd:
                code, output = run_restart_command_fn(cfg.restart_mcp_cmd, cfg.project_root)
                restart_events.append(
                    {"round": round_index, "return_code": code, "output_tail": output[-400:]}
                )
                if code != 0:
                    raise RuntimeError(
                        f"restart command failed at round {round_index} (code={code}): "
                        f"{output[-400:]}"
                    )
                if cfg.restart_mcp_settle_secs > 0:
                    time.sleep(cfg.restart_mcp_settle_secs)

            round_started = time.monotonic()
            with concurrent.futures.ThreadPoolExecutor(max_workers=cfg.parallel) as executor:
                futures = [
                    executor.submit(
                        run_single_probe_fn,
                        cfg,
                        round_index,
                        worker_index,
                        probe_result_cls=probe_result_cls,
                    )
                    for worker_index in range(1, cfg.parallel + 1)
                ]
                for future in concurrent.futures.as_completed(futures):
                    results.append(future.result())

            if cfg.cooldown_secs > 0 and round_index < cfg.rounds:
                elapsed = time.monotonic() - round_started
                if elapsed < cfg.cooldown_secs:
                    time.sleep(cfg.cooldown_secs - elapsed)
    finally:
        stop_health_sampler(
            cfg,
            health_stop=health_stop,
            health_thread=health_thread,
        )
    with health_samples_lock:
        health_rows = list(health_samples)
    summary = summarize_fn(results, health_rows)
    return build_run_report(
        cfg,
        started_dt=started_dt,
        started_monotonic=started,
        health_preflight=health_preflight,
        restart_events=restart_events,
        summary=summary,
        results=results,
        health_rows=health_rows,
    )
