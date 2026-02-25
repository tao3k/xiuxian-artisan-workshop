#!/usr/bin/env python3
"""Execution helpers for MCP startup stress flow."""

from __future__ import annotations

import concurrent.futures
import threading
import time
from dataclasses import asdict
from datetime import UTC, datetime
from typing import Any


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
    health_preflight = None

    if cfg.health_url:
        ok, detail = check_health_fn(cfg.health_url)
        health_preflight = {"url": cfg.health_url, "ok": ok, "detail": detail}
        if cfg.strict_health_check and not ok:
            raise RuntimeError(f"health check failed before stress: {detail}")

    results: list[Any] = []
    health_samples: list[Any] = []
    health_samples_lock = threading.Lock()
    health_stop = threading.Event()
    restart_events: list[dict[str, object]] = []

    health_thread: threading.Thread | None = None
    if cfg.health_url and cfg.health_probe_interval_secs > 0:
        health_url = cfg.health_url

        def _health_loop() -> None:
            assert health_url is not None
            while not health_stop.is_set():
                sample = collect_health_sample_fn(
                    health_url,
                    cfg.health_probe_timeout_secs,
                    health_sample_cls=health_sample_cls,
                )
                with health_samples_lock:
                    health_samples.append(sample)
                if cfg.health_probe_interval_secs <= 0:
                    return
                health_stop.wait(cfg.health_probe_interval_secs)

        health_thread = threading.Thread(target=_health_loop, daemon=True)
        health_thread.start()

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
        health_stop.set()
        if health_thread is not None:
            health_thread.join(timeout=max(1.0, cfg.health_probe_timeout_secs + 1.0))

    finished_dt = datetime.now(UTC)
    with health_samples_lock:
        health_rows = list(health_samples)
    summary = summarize_fn(results, health_rows)
    return {
        "started_at": started_dt.isoformat(),
        "finished_at": finished_dt.isoformat(),
        "duration_ms": int((time.monotonic() - started) * 1000),
        "config": {
            "rounds": cfg.rounds,
            "parallel": cfg.parallel,
            "startup_timeout_secs": cfg.startup_timeout_secs,
            "cooldown_secs": cfg.cooldown_secs,
            "executable": str(cfg.executable),
            "mcp_config": str(cfg.mcp_config),
            "bind_addr": cfg.bind_addr,
            "rust_log": cfg.rust_log,
            "health_url": cfg.health_url,
            "health_probe_interval_secs": cfg.health_probe_interval_secs,
            "health_probe_timeout_secs": cfg.health_probe_timeout_secs,
            "restart_mcp_cmd": cfg.restart_mcp_cmd,
            "restart_mcp_settle_secs": cfg.restart_mcp_settle_secs,
        },
        "health_preflight": health_preflight,
        "restart_events": restart_events,
        "summary": summary,
        "results": [asdict(row) for row in results],
        "health_samples": [asdict(row) for row in health_rows[-200:]],
    }
