#!/usr/bin/env python3
"""Compatibility facade for MCP startup suite quality gate helpers."""

from __future__ import annotations

from mcp_startup_suite_quality_baseline import (
    load_baseline_mode_p95s as _load_baseline_mode_p95s_impl,
)
from mcp_startup_suite_quality_baseline import (
    mode_failed as _mode_failed_impl,
)
from mcp_startup_suite_quality_baseline import (
    mode_p95 as _mode_p95_impl,
)
from mcp_startup_suite_quality_gates import (
    evaluate_quality_gates as _evaluate_quality_gates_impl,
)


def mode_p95(summary: dict[str, object]) -> float:
    """Extract mode startup p95 metric from mode summary."""
    return _mode_p95_impl(summary)


def mode_failed(summary: dict[str, object]) -> int:
    """Extract failed probe count from mode summary."""
    return _mode_failed_impl(summary)


def load_baseline_mode_p95s(path):
    """Load baseline report and collect per-mode p95 values."""
    return _load_baseline_mode_p95s_impl(path)


def evaluate_quality_gates(cfg, modes):
    """Evaluate quality gates for all executed startup modes."""
    return _evaluate_quality_gates_impl(cfg, modes)
