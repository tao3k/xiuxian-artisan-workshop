#!/usr/bin/env python3
"""Baseline-loading helpers for MCP startup suite quality gates."""

from __future__ import annotations

import json
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def mode_p95(summary: dict[str, object]) -> float:
    """Extract mode startup p95 metric from mode summary."""
    return float(summary.get("success_p95_startup_ms", 0.0))


def mode_failed(summary: dict[str, object]) -> int:
    """Extract failed probe count from mode summary."""
    return int(summary.get("failed", 0))


def load_baseline_mode_p95s(path: Path) -> dict[str, float]:
    """Load baseline report and collect per-mode p95 values."""
    payload = json.loads(path.read_text(encoding="utf-8"))
    modes = payload.get("modes")
    if not isinstance(modes, list):
        return {}
    result: dict[str, float] = {}
    for mode in modes:
        if not isinstance(mode, dict):
            continue
        mode_name = mode.get("mode")
        summary = mode.get("summary")
        if isinstance(mode_name, str) and isinstance(summary, dict):
            result[mode_name] = mode_p95(summary)
    return result
