#!/usr/bin/env python3
"""Unit tests for memory CI gate config validation."""

from __future__ import annotations

import argparse

import pytest
from memory_ci_gate_config_validation import validate_args


def build_args(**overrides: object) -> argparse.Namespace:
    baseline = {
        "valkey_port": 6379,
        "webhook_port": 18081,
        "telegram_api_port": 18080,
        "min_session_steps": 20,
        "cross_group_max_wait": 90,
        "cross_group_max_idle": 80,
        "cross_group_max_parallel": 3,
        "slow_response_min_duration_ms": 20000,
        "slow_response_long_step_ms": 1200,
        "slow_response_min_long_steps": 1,
        "discover_cache_hit_p95_ms": 15.0,
        "discover_cache_miss_p95_ms": 80.0,
        "discover_cache_bench_iterations": 12,
        "max_mcp_call_waiting_events": 0,
        "max_mcp_connect_waiting_events": 0,
        "max_mcp_waiting_events_total": 0,
        "max_memory_stream_read_failed_events": 0,
        "max_embedding_timeout_fallback_turns": 0,
        "max_embedding_cooldown_fallback_turns": 0,
        "max_embedding_unavailable_fallback_turns": 0,
        "max_embedding_fallback_turns_total": 0,
        "trace_min_quality_score": 90.0,
        "trace_max_events": 2000,
        "cross_group_scenario": "cross_group_control_plane_stress",
    }
    baseline.update(overrides)
    return argparse.Namespace(**baseline)


def test_validate_args_accepts_baseline() -> None:
    validate_args(build_args())


@pytest.mark.parametrize(
    ("field", "value", "pattern"),
    [
        ("valkey_port", 0, "--valkey-port must be in range 1..65535"),
        ("webhook_port", 65536, "--webhook-port must be in range 1..65535"),
        ("telegram_api_port", -1, "--telegram-api-port must be in range 1..65535"),
        ("cross_group_max_parallel", 0, "--cross-group-max-parallel must be a positive integer."),
        ("trace_max_events", 0, "--trace-max-events must be a positive integer."),
    ],
)
def test_validate_args_rejects_invalid_numeric_ranges(
    field: str, value: object, pattern: str
) -> None:
    with pytest.raises(ValueError, match=pattern):
        validate_args(build_args(**{field: value}))


def test_validate_args_rejects_empty_cross_group_scenario() -> None:
    with pytest.raises(ValueError, match=r"--cross-group-scenario must not be empty\."):
        validate_args(build_args(cross_group_scenario=" "))
