#!/usr/bin/env python3
"""Quality-gate wrappers for memory CI runtime."""

from __future__ import annotations

import importlib
from typing import Any

_gates_module = importlib.import_module("memory_ci_gate_runtime_gates")


def run_reflection_quality_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
) -> None:
    """Run reflection-quality Rust regression gate."""
    _gates_module.run_reflection_quality_gate(
        cfg,
        cwd=cwd,
        env=env,
        run_command_fn=run_command_fn,
    )


def run_discover_cache_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
) -> None:
    """Run discover-cache latency gate."""
    _gates_module.run_discover_cache_gate(
        cfg,
        cwd=cwd,
        env=env,
        run_command_fn=run_command_fn,
    )


def run_trace_reconstruction_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
    assert_trace_reconstruction_quality_fn: Any,
) -> None:
    """Run trace reconstruction gate and validate resulting report."""
    _gates_module.run_trace_reconstruction_gate(
        cfg,
        cwd=cwd,
        env=env,
        run_command_fn=run_command_fn,
        assert_trace_reconstruction_quality_fn=assert_trace_reconstruction_quality_fn,
    )


def run_cross_group_complex_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
    assert_cross_group_complex_quality_fn: Any,
) -> None:
    """Run cross-group mixed-concurrency scenario gate."""
    _gates_module.run_cross_group_complex_gate(
        cfg,
        cwd=cwd,
        env=env,
        run_command_fn=run_command_fn,
        assert_cross_group_complex_quality_fn=assert_cross_group_complex_quality_fn,
    )
