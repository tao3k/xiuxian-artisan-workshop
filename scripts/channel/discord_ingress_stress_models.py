#!/usr/bin/env python3
"""Datamodels for Discord ingress stress probe."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


@dataclass(frozen=True)
class StressConfig:
    """Validated runtime configuration for Discord ingress stress probe."""

    rounds: int
    warmup_rounds: int
    parallel: int
    requests_per_worker: int
    timeout_secs: float
    cooldown_secs: float
    ingress_url: str
    log_file: Path
    secret_token: str | None
    channel_id: str
    user_id: str
    guild_id: str | None
    username: str | None
    role_ids: tuple[str, ...]
    prompt: str
    output_json: Path
    output_markdown: Path
    quality_max_failure_rate: float
    quality_max_p95_ms: float | None
    quality_min_rps: float | None


@dataclass(frozen=True)
class RoundResult:
    """One round aggregation result."""

    round_index: int
    warmup: bool
    total_requests: int
    success_requests: int
    failed_requests: int
    non_200_responses: int
    responses_5xx: int
    connection_errors: int
    avg_latency_ms: float
    p95_latency_ms: float
    max_latency_ms: float
    duration_ms: int
    rps: float
    log_parsed_messages: int
    log_queue_wait_events: int
    log_foreground_gate_wait_events: int
    log_inbound_queue_unavailable_events: int
