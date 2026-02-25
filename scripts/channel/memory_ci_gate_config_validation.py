#!/usr/bin/env python3
"""Argument validation for memory CI gate config."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse


def validate_args(args: argparse.Namespace) -> None:
    """Validate parsed CLI args for memory CI gate."""
    if args.valkey_port <= 0 or args.valkey_port > 65535:
        raise ValueError("--valkey-port must be in range 1..65535")
    if args.webhook_port <= 0 or args.webhook_port > 65535:
        raise ValueError("--webhook-port must be in range 1..65535")
    if args.telegram_api_port <= 0 or args.telegram_api_port > 65535:
        raise ValueError("--telegram-api-port must be in range 1..65535")
    if args.min_session_steps <= 0:
        raise ValueError("--min-session-steps must be a positive integer.")
    if args.cross_group_max_wait <= 0:
        raise ValueError("--cross-group-max-wait must be a positive integer.")
    if args.cross_group_max_idle <= 0:
        raise ValueError("--cross-group-max-idle must be a positive integer.")
    if args.cross_group_max_parallel <= 0:
        raise ValueError("--cross-group-max-parallel must be a positive integer.")
    if args.slow_response_min_duration_ms <= 0:
        raise ValueError("--slow-response-min-duration-ms must be a positive integer.")
    if args.slow_response_long_step_ms <= 0:
        raise ValueError("--slow-response-long-step-ms must be a positive integer.")
    if args.slow_response_min_long_steps <= 0:
        raise ValueError("--slow-response-min-long-steps must be a positive integer.")
    if args.discover_cache_hit_p95_ms <= 0:
        raise ValueError("--discover-cache-hit-p95-ms must be positive.")
    if args.discover_cache_miss_p95_ms <= 0:
        raise ValueError("--discover-cache-miss-p95-ms must be positive.")
    if args.discover_cache_bench_iterations <= 0:
        raise ValueError("--discover-cache-bench-iterations must be a positive integer.")
    if args.max_mcp_call_waiting_events < 0:
        raise ValueError("--max-mcp-call-waiting-events must be >= 0.")
    if args.max_mcp_connect_waiting_events < 0:
        raise ValueError("--max-mcp-connect-waiting-events must be >= 0.")
    if args.max_mcp_waiting_events_total < 0:
        raise ValueError("--max-mcp-waiting-events-total must be >= 0.")
    if args.max_memory_stream_read_failed_events < 0:
        raise ValueError("--max-memory-stream-read-failed-events must be >= 0.")
    if args.max_embedding_timeout_fallback_turns < 0:
        raise ValueError("--max-embedding-timeout-fallback-turns must be >= 0.")
    if args.max_embedding_cooldown_fallback_turns < 0:
        raise ValueError("--max-embedding-cooldown-fallback-turns must be >= 0.")
    if args.max_embedding_unavailable_fallback_turns < 0:
        raise ValueError("--max-embedding-unavailable-fallback-turns must be >= 0.")
    if args.max_embedding_fallback_turns_total < 0:
        raise ValueError("--max-embedding-fallback-turns-total must be >= 0.")
    if args.trace_min_quality_score <= 0:
        raise ValueError("--trace-min-quality-score must be positive.")
    if args.trace_max_events <= 0:
        raise ValueError("--trace-max-events must be a positive integer.")
    if not args.cross_group_scenario.strip():
        raise ValueError("--cross-group-scenario must not be empty.")
