#!/usr/bin/env python3
"""Shared quality-gate runners for memory CI gate profiles."""

from __future__ import annotations

from typing import Any


def run_common_post_gates(
    cfg: Any,
    *,
    env: dict[str, str],
    run_reflection_quality_gate_fn: Any,
    run_discover_cache_gate_fn: Any,
    run_trace_reconstruction_gate_fn: Any,
    assert_mcp_waiting_warning_budget_fn: Any,
    assert_memory_stream_warning_budget_fn: Any,
) -> None:
    """Run common post-suite quality/warning gates."""
    run_reflection_quality_gate_fn(cfg, cwd=cfg.project_root, env=env)
    run_discover_cache_gate_fn(cfg, cwd=cfg.project_root, env=env)
    run_trace_reconstruction_gate_fn(cfg, cwd=cfg.project_root, env=env)
    assert_mcp_waiting_warning_budget_fn(cfg)
    assert_memory_stream_warning_budget_fn(cfg)
