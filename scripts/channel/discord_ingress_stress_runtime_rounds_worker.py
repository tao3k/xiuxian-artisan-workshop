#!/usr/bin/env python3
"""Worker execution logic for Discord ingress stress rounds."""

from __future__ import annotations

from typing import Any


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
