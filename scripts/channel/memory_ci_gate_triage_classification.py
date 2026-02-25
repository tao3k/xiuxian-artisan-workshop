#!/usr/bin/env python3
"""Failure classification helpers for memory CI gate triage."""

from __future__ import annotations


def is_gate_step_error(error: Exception) -> bool:
    """Return whether an exception has gate-step attributes."""
    return all(hasattr(error, attr) for attr in ("title", "cmd", "returncode"))


def classify_gate_failure(error: Exception) -> tuple[str, str]:
    """Classify a gate failure into category + short summary."""
    if is_gate_step_error(error):
        title = str(getattr(error, "title", "")).lower()
        if "start omni-agent webhook runtime" in title:
            return ("runtime_startup_process", "runtime startup command failed")
        if "memory suite" in title:
            return ("memory_suite_subprocess", "memory suite subprocess failed")
        if "session matrix" in title:
            return ("session_matrix_subprocess", "session matrix subprocess failed")
        if "cross-group mixed-concurrency" in title:
            return ("cross_group_subprocess", "cross-group mixed-concurrency subprocess failed")
        if "memory a/b benchmark" in title:
            return ("benchmark_subprocess", "memory benchmark subprocess failed")
        if "reflection quality gate" in title:
            return ("reflection_gate_subprocess", "reflection quality cargo gate failed")
        if "discover cache latency gate" in title:
            return ("discover_cache_gate_subprocess", "discover cache cargo gate failed")
        if "trace reconstruction gate" in title:
            return ("trace_reconstruction_subprocess", "trace reconstruction subprocess failed")

    message = str(error).lower()
    if (
        "timed out waiting for log pattern" in message
        and "telegram webhook listening on" in message
    ):
        return ("runtime_startup_timeout", "runtime did not become ready before timeout")
    if "runtime process exited before readiness check passed" in message:
        return ("runtime_startup_process", "runtime exited before readiness check passed")
    if "evolution quality gates failed" in message:
        return ("evolution_quality", "evolution quality thresholds failed")
    if "slow-response resilience gate failed" in message:
        return ("slow_response_quality", "slow-response resilience thresholds failed")
    if "session matrix report indicates overall failure" in message:
        return ("session_matrix_quality", "session matrix report indicates failure")
    if "session matrix" in message and "failed" in message:
        return ("session_matrix_quality", "session matrix quality gate failed")
    if "cross-group complex report indicates overall failure" in message:
        return ("cross_group_quality", "cross-group report indicates failure")
    if "cross-group complex scenario failed" in message:
        return ("cross_group_quality", "cross-group scenario quality gate failed")
    if "benchmark quality gates failed" in message:
        return ("benchmark_quality", "benchmark quality thresholds failed")
    if "trace reconstruction quality gates failed" in message:
        return ("trace_reconstruction_quality", "trace reconstruction quality gate failed")
    if "mcp waiting warning budget exceeded" in message:
        return ("mcp_waiting_budget", "mcp waiting warning budget exceeded")
    if "memory stream warning budget exceeded" in message:
        return ("memory_stream_budget", "memory stream warning budget exceeded")
    return ("unknown", "unclassified gate failure")
