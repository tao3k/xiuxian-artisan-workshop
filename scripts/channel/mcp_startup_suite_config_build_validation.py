#!/usr/bin/env python3
"""Validation helpers for MCP startup suite config construction."""

from __future__ import annotations

from typing import Any


def validate_numeric_constraints(args: Any) -> None:
    """Validate numeric CLI constraints for suite config build."""
    if args.hot_rounds <= 0 or args.hot_parallel <= 0:
        raise ValueError("--hot-rounds and --hot-parallel must be positive.")
    if args.cold_rounds <= 0 or args.cold_parallel <= 0:
        raise ValueError("--cold-rounds and --cold-parallel must be positive.")
    if args.startup_timeout_secs <= 0:
        raise ValueError("--startup-timeout-secs must be positive.")
    if args.cooldown_secs < 0:
        raise ValueError("--cooldown-secs must be >= 0.")
    if args.restart_mcp_settle_secs < 0:
        raise ValueError("--restart-mcp-settle-secs must be >= 0.")
    if args.health_probe_interval_secs < 0:
        raise ValueError("--health-probe-interval-secs must be >= 0.")
    if args.health_probe_timeout_secs <= 0:
        raise ValueError("--health-probe-timeout-secs must be positive.")
    if args.restart_health_timeout_secs <= 0:
        raise ValueError("--restart-health-timeout-secs must be positive.")
    if args.mcp_port <= 0:
        raise ValueError("--mcp-port must be positive.")
    if args.quality_max_failed_probes < 0:
        raise ValueError("--quality-max-failed-probes must be >= 0.")
    if args.quality_max_hot_p95_ms <= 0:
        raise ValueError("--quality-max-hot-p95-ms must be positive.")
    if args.quality_max_cold_p95_ms <= 0:
        raise ValueError("--quality-max-cold-p95-ms must be positive.")
    if args.quality_min_health_samples < 0:
        raise ValueError("--quality-min-health-samples must be >= 0.")
    if args.quality_max_health_failure_rate < 0:
        raise ValueError("--quality-max-health-failure-rate must be >= 0.")
    if args.quality_max_health_p95_ms <= 0:
        raise ValueError("--quality-max-health-p95-ms must be positive.")
    if args.quality_max_hot_p95_regression_ratio < 0:
        raise ValueError("--quality-max-hot-p95-regression-ratio must be >= 0.")
    if args.quality_max_cold_p95_regression_ratio < 0:
        raise ValueError("--quality-max-cold-p95-regression-ratio must be >= 0.")
