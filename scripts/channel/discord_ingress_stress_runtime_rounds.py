#!/usr/bin/env python3
"""Round execution helpers for Discord ingress stress runtime."""

from __future__ import annotations

import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Any


def collect_log_stats(lines: list[str]) -> dict[str, int]:
    """Extract queue-pressure and parse counters from log lines."""

    def _count(token: str) -> int:
        return sum(1 for line in lines if token in line)

    return {
        "parsed_messages": _count("discord ingress parsed message"),
        "queue_wait_events": _count('event="discord.ingress.inbound_queue_wait"'),
        "foreground_gate_wait_events": _count('event="discord.foreground.gate_wait"'),
        "inbound_queue_unavailable_events": _count("discord inbound queue unavailable"),
    }


def run_worker(
    cfg: Any,
    round_index: int,
    worker_index: int,
    *,
    next_event_id_fn: Any,
    build_ingress_payload_fn: Any,
    post_ingress_event_fn: Any,
) -> dict[str, Any]:
    """Run one worker burst for a stress round."""
    success_requests = 0
    failed_requests = 0
    non_200_responses = 0
    responses_5xx = 0
    connection_errors = 0
    latencies: list[float] = []

    for request_index in range(cfg.requests_per_worker):
        event_id = next_event_id_fn()
        prompt = f"{cfg.prompt} [round={round_index} worker={worker_index} req={request_index}]"
        payload = build_ingress_payload_fn(cfg, event_id, prompt)
        status, _body, latency_ms = post_ingress_event_fn(
            cfg.ingress_url,
            payload,
            cfg.secret_token,
            cfg.timeout_secs,
        )
        latencies.append(latency_ms)
        if status == 200:
            success_requests += 1
        else:
            failed_requests += 1
            if status == 0:
                connection_errors += 1
            else:
                non_200_responses += 1
                if 500 <= status <= 599:
                    responses_5xx += 1

    return {
        "total_requests": cfg.requests_per_worker,
        "success_requests": success_requests,
        "failed_requests": failed_requests,
        "non_200_responses": non_200_responses,
        "responses_5xx": responses_5xx,
        "connection_errors": connection_errors,
        "latencies_ms": tuple(latencies),
    }


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

    latencies: list[float] = []
    total_requests = 0
    success_requests = 0
    failed_requests = 0
    non_200_responses = 0
    responses_5xx = 0
    connection_errors = 0

    for result in worker_results:
        total_requests += int(result["total_requests"])
        success_requests += int(result["success_requests"])
        failed_requests += int(result["failed_requests"])
        non_200_responses += int(result["non_200_responses"])
        responses_5xx += int(result["responses_5xx"])
        connection_errors += int(result["connection_errors"])
        latencies.extend(result["latencies_ms"])

    avg_latency_ms = sum(latencies) / len(latencies) if latencies else 0.0
    p95_latency_ms = p95_fn(latencies)
    max_latency_ms = max(latencies) if latencies else 0.0
    duration_secs = max(duration_ms / 1000.0, 0.001)
    rps = total_requests / duration_secs

    return round_result_cls(
        round_index=round_index,
        warmup=warmup,
        total_requests=total_requests,
        success_requests=success_requests,
        failed_requests=failed_requests,
        non_200_responses=non_200_responses,
        responses_5xx=responses_5xx,
        connection_errors=connection_errors,
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
