#!/usr/bin/env python3
"""Worker-result aggregation for Discord ingress stress rounds."""

from __future__ import annotations

from typing import Any


def aggregate_worker_results(worker_results: list[dict[str, Any]]) -> dict[str, Any]:
    """Aggregate worker-level counters and latency arrays."""
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

    return {
        "total_requests": total_requests,
        "success_requests": success_requests,
        "failed_requests": failed_requests,
        "non_200_responses": non_200_responses,
        "responses_5xx": responses_5xx,
        "connection_errors": connection_errors,
        "latencies_ms": latencies,
    }
