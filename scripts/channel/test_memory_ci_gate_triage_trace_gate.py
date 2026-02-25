#!/usr/bin/env python3
"""Trace reconstruction gate tests for memory CI triage."""

from __future__ import annotations

from dataclasses import replace
from typing import Any

import test_omni_agent_memory_ci_gate as gate_module
from test_memory_ci_gate import build_cfg
from test_omni_agent_memory_ci_gate import run_trace_reconstruction_gate


def test_run_trace_reconstruction_gate_nightly_requires_injection_mode(
    monkeypatch: Any, tmp_path
) -> None:
    cfg = build_cfg(tmp_path)
    script = cfg.script_dir / "reconstruct_omni_agent_trace.py"
    script.write_text("print('noop')\n", encoding="utf-8")

    captured_cmds: list[list[str]] = []

    def _fake_run_command(
        cmd: list[str],
        *,
        title,
        cwd,
        env,
        check: bool = True,
        capture_output: bool = False,
    ) -> None:
        del title, cwd, env, check, capture_output
        captured_cmds.append(cmd)

    monkeypatch.setattr(gate_module, "run_command", _fake_run_command)
    monkeypatch.setattr(gate_module, "assert_trace_reconstruction_quality", lambda _cfg: None)

    run_trace_reconstruction_gate(cfg, cwd=tmp_path, env={})

    assert captured_cmds, "trace reconstruction gate should invoke run_command"
    command = captured_cmds[0]
    stages = [
        command[index + 1]
        for index in range(len(command) - 1)
        if command[index] == "--required-stage"
    ]
    assert stages == ["route", "injection", "injection_mode", "reflection", "memory"]


def test_run_trace_reconstruction_gate_quick_requires_memory_only(
    monkeypatch: Any, tmp_path
) -> None:
    cfg = replace(build_cfg(tmp_path), profile="quick")
    script = cfg.script_dir / "reconstruct_omni_agent_trace.py"
    script.write_text("print('noop')\n", encoding="utf-8")

    captured_cmds: list[list[str]] = []

    def _fake_run_command(
        cmd: list[str],
        *,
        title,
        cwd,
        env,
        check: bool = True,
        capture_output: bool = False,
    ) -> None:
        del title, cwd, env, check, capture_output
        captured_cmds.append(cmd)

    monkeypatch.setattr(gate_module, "run_command", _fake_run_command)
    monkeypatch.setattr(gate_module, "assert_trace_reconstruction_quality", lambda _cfg: None)

    run_trace_reconstruction_gate(cfg, cwd=tmp_path, env={})

    assert captured_cmds, "trace reconstruction gate should invoke run_command"
    command = captured_cmds[0]
    stages = [
        command[index + 1]
        for index in range(len(command) - 1)
        if command[index] == "--required-stage"
    ]
    assert stages == ["memory"]
