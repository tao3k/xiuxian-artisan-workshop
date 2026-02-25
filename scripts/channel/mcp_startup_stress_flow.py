#!/usr/bin/env python3
"""Compatibility facade for MCP startup stress flow orchestration."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from mcp_startup_stress_flow_run import run_stress as _run_stress_impl
from mcp_startup_stress_flow_summary import summarize as _summarize_impl

if TYPE_CHECKING:
    from collections.abc import Iterable


def summarize(
    results: Iterable[Any],
    health_samples: Iterable[Any],
    *,
    p95_fn: Any,
    summarize_health_samples_fn: Any,
) -> dict[str, object]:
    """Summarize probe results and health telemetry."""
    return _summarize_impl(
        results,
        health_samples,
        p95_fn=p95_fn,
        summarize_health_samples_fn=summarize_health_samples_fn,
    )


def run_stress(
    cfg: Any,
    *,
    probe_result_cls: Any,
    health_sample_cls: Any,
    check_health_fn: Any,
    run_restart_command_fn: Any,
    collect_health_sample_fn: Any,
    run_single_probe_fn: Any,
    summarize_fn: Any,
) -> dict[str, object]:
    """Execute full stress run including optional restart and health sampling."""
    return _run_stress_impl(
        cfg,
        probe_result_cls=probe_result_cls,
        health_sample_cls=health_sample_cls,
        check_health_fn=check_health_fn,
        run_restart_command_fn=run_restart_command_fn,
        collect_health_sample_fn=collect_health_sample_fn,
        run_single_probe_fn=run_single_probe_fn,
        summarize_fn=summarize_fn,
    )


__all__ = ["run_stress", "summarize"]
