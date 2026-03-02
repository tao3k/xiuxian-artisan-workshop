#!/usr/bin/env python3
"""Report payload builders for Discord ingress stress runtime summaries."""

from __future__ import annotations

from typing import Any


def build_report(
    cfg: Any,
    *,
    started_at: str,
    finished_at: str,
    duration_ms: int,
    rounds: list[Any],
    measured: list[Any],
    quality_passed: bool,
    quality_failures: list[str],
) -> dict[str, object]:
    """Build final structured stress report payload."""
    total_requests = sum(int(row.total_requests) for row in measured)
    success_requests = sum(int(row.success_requests) for row in measured)
    failed_requests = sum(int(row.failed_requests) for row in measured)
    failure_rate = (failed_requests / total_requests) if total_requests > 0 else 1.0

    avg_rps = sum(float(row.rps) for row in measured) / len(measured) if measured else 0.0
    measured_p95 = max((float(row.p95_latency_ms) for row in measured), default=0.0)
    queue_wait_events = sum(int(row.log_queue_wait_events) for row in measured)
    gate_wait_events = sum(int(row.log_foreground_gate_wait_events) for row in measured)
    queue_unavailable_events = sum(
        int(row.log_inbound_queue_unavailable_events) for row in measured
    )
    parsed_messages = sum(int(row.log_parsed_messages) for row in measured)

    return {
        "started_at": started_at,
        "finished_at": finished_at,
        "duration_ms": duration_ms,
        "inputs": {
            "rounds": cfg.rounds,
            "warmup_rounds": cfg.warmup_rounds,
            "parallel": cfg.parallel,
            "requests_per_worker": cfg.requests_per_worker,
            "timeout_secs": cfg.timeout_secs,
            "cooldown_secs": cfg.cooldown_secs,
            "ingress_url": cfg.ingress_url,
            "channel_id": cfg.channel_id,
            "user_id": cfg.user_id,
            "guild_id": cfg.guild_id,
            "username": cfg.username,
            "role_ids": list(cfg.role_ids),
            "log_file": str(cfg.log_file),
            "quality_max_failure_rate": cfg.quality_max_failure_rate,
            "quality_max_p95_ms": cfg.quality_max_p95_ms,
            "quality_min_rps": cfg.quality_min_rps,
        },
        "summary": {
            "measured_rounds": len(measured),
            "total_requests": total_requests,
            "success_requests": success_requests,
            "failed_requests": failed_requests,
            "failure_rate": failure_rate,
            "average_rps": avg_rps,
            "max_round_p95_ms": measured_p95,
            "parsed_messages": parsed_messages,
            "queue_wait_events": queue_wait_events,
            "foreground_gate_wait_events": gate_wait_events,
            "inbound_queue_unavailable_events": queue_unavailable_events,
            "quality_passed": quality_passed,
            "quality_failures": quality_failures,
        },
        "rounds": [
            {
                "round_index": row.round_index,
                "warmup": row.warmup,
                "total_requests": row.total_requests,
                "success_requests": row.success_requests,
                "failed_requests": row.failed_requests,
                "non_200_responses": row.non_200_responses,
                "responses_5xx": row.responses_5xx,
                "connection_errors": row.connection_errors,
                "avg_latency_ms": row.avg_latency_ms,
                "p95_latency_ms": row.p95_latency_ms,
                "max_latency_ms": row.max_latency_ms,
                "duration_ms": row.duration_ms,
                "rps": row.rps,
                "log_parsed_messages": row.log_parsed_messages,
                "log_queue_wait_events": row.log_queue_wait_events,
                "log_foreground_gate_wait_events": row.log_foreground_gate_wait_events,
                "log_inbound_queue_unavailable_events": row.log_inbound_queue_unavailable_events,
            }
            for row in rounds
        ],
    }
