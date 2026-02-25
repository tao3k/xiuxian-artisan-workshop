#!/usr/bin/env python3
"""Reflection-quality gate helper for memory CI runtime."""

from __future__ import annotations

from typing import Any


def run_reflection_quality_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
) -> None:
    """Run reflection-quality Rust regression gate."""
    if cfg.skip_reflection_quality_gate:
        print("Skipping reflection quality gate (--skip-reflection-quality-gate).", flush=True)
        return
    run_command_fn(
        [
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--lib",
            "reflective_runtime_long_horizon_quality_thresholds",
        ],
        title="Reflection quality gate: long-horizon policy hint thresholds",
        cwd=cwd,
        env=env,
    )
