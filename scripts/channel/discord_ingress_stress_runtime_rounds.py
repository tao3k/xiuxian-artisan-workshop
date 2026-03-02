#!/usr/bin/env python3
"""Round execution helpers for Discord ingress stress runtime."""

from __future__ import annotations

import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Any

import discord_ingress_stress_runtime_rounds_worker as _worker_module
from discord_ingress_stress_runtime_rounds_aggregate import aggregate_worker_results
from discord_ingress_stress_runtime_rounds_log_stats import collect_log_stats

run_worker = _worker_module.run_worker


def run_round(
    cfg: Any,
    round_index: int,
    *,
    warmup: bool,
    round_result_cls: Any,
    init_log_offset_fn: Any,
    read_new_log_lines_fn: Any,
    run_worker_fn: Any,
    p95_fn: Any,
) -> Any:
    """Execute one round and return typed round aggregation result."""
    log_offset = init_log_offset_fn(cfg.log_file)
    round_started = time.perf_counter()

    worker_results: list[dict[str, Any]] = []
    with ThreadPoolExecutor(max_workers=cfg.parallel) as pool:
        futures = [pool.submit(run_worker_fn, cfg, round_index, i + 1) for i in range(cfg.parallel)]
        for future in as_completed(futures):
            worker_results.append(future.result())

    duration_ms = int((time.perf_counter() - round_started) * 1000)
    _, log_lines = read_new_log_lines_fn(cfg.log_file, log_offset)
    log_stats = collect_log_stats(log_lines)
    aggregates = aggregate_worker_results(worker_results)
    latencies = aggregates["latencies_ms"]
    avg_latency_ms = sum(latencies) / len(latencies) if latencies else 0.0
    p95_latency_ms = p95_fn(latencies)
    max_latency_ms = max(latencies) if latencies else 0.0
    rps = aggregates["total_requests"] / max(duration_ms / 1000.0, 0.001)

    return round_result_cls(
        round_index=round_index,
        warmup=warmup,
        total_requests=aggregates["total_requests"],
        success_requests=aggregates["success_requests"],
        failed_requests=aggregates["failed_requests"],
        non_200_responses=aggregates["non_200_responses"],
        responses_5xx=aggregates["responses_5xx"],
        connection_errors=aggregates["connection_errors"],
        avg_latency_ms=round(avg_latency_ms, 3),
        p95_latency_ms=round(p95_latency_ms, 3),
        max_latency_ms=round(max_latency_ms, 3),
        duration_ms=duration_ms,
        rps=round(rps, 3),
        log_parsed_messages=log_stats["parsed_messages"],
        log_queue_wait_events=log_stats["queue_wait_events"],
        log_foreground_gate_wait_events=log_stats["foreground_gate_wait_events"],
        log_inbound_queue_unavailable_events=log_stats["inbound_queue_unavailable_events"],
    )
