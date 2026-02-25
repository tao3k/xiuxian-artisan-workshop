#!/usr/bin/env python3
"""Failure classification and repro command tests for memory CI gate triage."""

from __future__ import annotations

from test_memory_ci_gate import build_cfg
from test_omni_agent_memory_ci_gate import (
    GateStepError,
    build_gate_failure_repro_commands,
    classify_gate_failure,
)


def test_classify_gate_failure_maps_waiting_budget_error() -> None:
    category, summary = classify_gate_failure(
        RuntimeError("mcp waiting warning budget exceeded: mcp_waiting_events_total=4 > 0")
    )
    assert category == "mcp_waiting_budget"
    assert "budget exceeded" in summary


def test_classify_gate_failure_maps_runtime_startup_exit() -> None:
    category, summary = classify_gate_failure(
        RuntimeError("runtime process exited before readiness check passed.")
    )
    assert category == "runtime_startup_process"
    assert "readiness" in summary


def test_build_gate_failure_repro_commands_includes_stage_command(tmp_path) -> None:
    cfg = build_cfg(tmp_path)
    stage_error = GateStepError(
        title="Discover cache latency gate (A3)",
        cmd=[
            "cargo",
            "test",
            "-p",
            "omni-agent",
            "--test",
            "mcp_discover_cache",
        ],
        returncode=101,
    )
    commands = build_gate_failure_repro_commands(
        cfg, category="discover_cache_gate_subprocess", error=stage_error
    )
    assert any(command.startswith("tail -n 200 ") for command in commands)
    assert any(
        "cargo test -p omni-agent --test mcp_discover_cache" in command for command in commands
    )
    assert any(
        "discover_calls_use_valkey_read_through_cache_when_configured" in command
        for command in commands
    )


def test_build_gate_failure_repro_commands_trace_quality_includes_injection_mode_stage(
    tmp_path,
) -> None:
    cfg = build_cfg(tmp_path)
    cfg.runtime_log_file.write_text(
        '2026-02-20T00:00:00Z INFO x: event="session.injection.snapshot_created"\n',
        encoding="utf-8",
    )
    commands = build_gate_failure_repro_commands(
        cfg,
        category="trace_reconstruction_quality",
        error=RuntimeError("trace reconstruction quality gates failed"),
    )
    trace_commands = [
        command for command in commands if "reconstruct_omni_agent_trace.py" in command
    ]
    assert trace_commands, "expected trace reconstruction repro command to be generated"
    assert any("--required-stage injection_mode" in command for command in trace_commands)
