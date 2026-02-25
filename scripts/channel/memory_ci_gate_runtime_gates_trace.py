#!/usr/bin/env python3
"""Trace reconstruction gate helper for memory CI runtime."""

from __future__ import annotations

import sys
from typing import Any


def run_trace_reconstruction_gate(
    cfg: Any,
    *,
    cwd: Any,
    env: dict[str, str],
    run_command_fn: Any,
    assert_trace_reconstruction_quality_fn: Any,
) -> None:
    """Run trace reconstruction gate and validate resulting report."""
    if cfg.skip_trace_reconstruction_gate:
        print("Skipping trace reconstruction gate (--skip-trace-reconstruction-gate).", flush=True)
        return

    script = cfg.script_dir / "reconstruct_omni_agent_trace.py"
    if not script.exists():
        raise FileNotFoundError(f"missing trace reconstruction script: {script}")

    required_stages = (
        ("route", "injection", "injection_mode", "reflection", "memory")
        if cfg.profile == "nightly"
        else ("memory",)
    )

    run_command_fn(
        [
            sys.executable,
            str(script),
            str(cfg.runtime_log_file),
            "--session-id",
            f"telegram:{cfg.chat_id}",
            "--max-events",
            str(cfg.trace_max_events),
            *[item for stage in required_stages for item in ("--required-stage", stage)],
            "--json-out",
            str(cfg.trace_report_json),
            "--markdown-out",
            str(cfg.trace_report_markdown),
        ],
        title="Trace reconstruction gate (S-01)",
        cwd=cwd,
        env=env,
    )
    assert_trace_reconstruction_quality_fn(cfg)
