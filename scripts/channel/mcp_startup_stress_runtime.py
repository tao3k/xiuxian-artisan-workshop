#!/usr/bin/env python3
"""Runtime helpers for MCP startup stress probe."""

from __future__ import annotations

import importlib
import subprocess
from typing import TYPE_CHECKING, Any

from mcp_startup_stress_flow import run_stress as _run_stress_flow
from mcp_startup_stress_flow import summarize as _summarize_flow
from mcp_startup_stress_probe import run_single_probe as _run_single_probe_impl

if TYPE_CHECKING:
    from collections.abc import Iterable

_health_module = importlib.import_module("mcp_startup_stress_health")


def check_health(url: str, timeout_secs: float = 2.0) -> tuple[bool, str]:
    """Issue one HTTP health check request."""
    return _health_module.check_health(url, timeout_secs=timeout_secs)


def run_restart_command(command: str, cwd: Any) -> tuple[int, str]:
    """Execute shell restart command and return exit code + merged output."""
    completed = subprocess.run(
        command,
        cwd=str(cwd),
        shell=True,
        capture_output=True,
        text=True,
        check=False,
    )
    output = (completed.stdout or "") + ("\n" + completed.stderr if completed.stderr else "")
    return completed.returncode, output.strip()


def classify_reason(
    *,
    ready_seen: bool,
    handshake_timeout_seen: bool,
    connect_failed_seen: bool,
    process_exited: bool,
    timed_out: bool,
) -> str:
    """Classify probe failure reason from runtime observations."""
    return _health_module.classify_reason(
        ready_seen=ready_seen,
        handshake_timeout_seen=handshake_timeout_seen,
        connect_failed_seen=connect_failed_seen,
        process_exited=process_exited,
        timed_out=timed_out,
    )


def p95(values: list[float]) -> float:
    """Compute p95 using index-based percentile for small sample stability."""
    return _health_module.p95(values)


def summarize_health_samples(samples: Iterable[Any]) -> dict[str, object]:
    """Summarize health samples into aggregate metrics."""
    return _health_module.summarize_health_samples(samples)


def collect_health_sample(url: str, timeout_secs: float, *, health_sample_cls: Any) -> Any:
    """Collect one typed health sample."""
    return _health_module.collect_health_sample(
        url,
        timeout_secs,
        health_sample_cls=health_sample_cls,
    )


def run_single_probe(
    cfg: Any, round_index: int, worker_index: int, *, probe_result_cls: Any
) -> Any:
    """Run one gateway startup probe and parse key handshake events."""
    return _run_single_probe_impl(
        cfg,
        round_index,
        worker_index,
        probe_result_cls=probe_result_cls,
        classify_reason_fn=classify_reason,
    )


def summarize(results: Iterable[Any], health_samples: Iterable[Any]) -> dict[str, object]:
    """Summarize probe results and health telemetry."""
    return _summarize_flow(
        results,
        health_samples,
        p95_fn=p95,
        summarize_health_samples_fn=summarize_health_samples,
    )


def run_stress(cfg: Any, *, probe_result_cls: Any, health_sample_cls: Any) -> dict[str, object]:
    """Execute full stress run including optional restart and health sampling."""
    return _run_stress_flow(
        cfg,
        probe_result_cls=probe_result_cls,
        health_sample_cls=health_sample_cls,
        check_health_fn=check_health,
        run_restart_command_fn=run_restart_command,
        collect_health_sample_fn=collect_health_sample,
        run_single_probe_fn=run_single_probe,
        summarize_fn=summarize,
    )
