#!/usr/bin/env python3
"""Compatibility facade for Discord ingress stress runtime summary helpers."""

from __future__ import annotations

from typing import Any

from discord_ingress_stress_runtime_summary_quality import (
    evaluate_quality as _evaluate_quality_impl,
)
from discord_ingress_stress_runtime_summary_report import build_report as _build_report_impl


def evaluate_quality(cfg: Any, measured_rounds: list[Any]) -> tuple[bool, list[str]]:
    """Evaluate pass/fail against configured quality thresholds."""
    return _evaluate_quality_impl(cfg, measured_rounds)


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
    return _build_report_impl(
        cfg,
        started_at=started_at,
        finished_at=finished_at,
        duration_ms=duration_ms,
        rounds=rounds,
        measured=measured,
        quality_passed=quality_passed,
        quality_failures=quality_failures,
    )
