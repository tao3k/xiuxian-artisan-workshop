#!/usr/bin/env python3
"""Unit tests for memory CI gate runner profile flows."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_profiles_module = importlib.import_module("memory_ci_gate_runner_profiles")


def test_run_quick_profile_executes_suite_and_quality_steps(tmp_path: Path) -> None:
    cfg = SimpleNamespace(
        quick_max_wait=20,
        quick_max_idle=15,
        username="tester",
        skip_rust_regressions=True,
        project_root=tmp_path,
    )
    env = {"VALKEY_URL": "redis://127.0.0.1:16379/0"}
    memory_suite = tmp_path / "test_omni_agent_memory_suite.py"
    recorded_titles: list[str] = []
    gate_steps: list[str] = []

    def _run_command(cmd: list[str], *, title: str, cwd: Path, env: dict[str, str]) -> None:
        assert cmd
        assert "--skip-rust" in cmd
        assert str(cwd) == str(tmp_path)
        assert env["VALKEY_URL"] == "redis://127.0.0.1:16379/0"
        recorded_titles.append(title)

    def _mark(name: str):
        def _inner(*_args: object, **_kwargs: object) -> None:
            gate_steps.append(name)

        return _inner

    _profiles_module.run_quick_profile(
        cfg,
        env=env,
        memory_suite=memory_suite,
        run_command_fn=_run_command,
        run_reflection_quality_gate_fn=_mark("reflection"),
        run_discover_cache_gate_fn=_mark("discover"),
        run_trace_reconstruction_gate_fn=_mark("trace"),
        assert_mcp_waiting_warning_budget_fn=_mark("mcp_wait"),
        assert_memory_stream_warning_budget_fn=_mark("memory_stream"),
    )

    assert recorded_titles == [
        "Quick gate: memory suite (black-box + Rust regressions, evolution skipped)"
    ]
    assert gate_steps == ["reflection", "discover", "trace", "mcp_wait", "memory_stream"]


def test_run_nightly_profile_runs_matrix_and_benchmark_flows(tmp_path: Path) -> None:
    cfg = SimpleNamespace(
        full_max_idle=25,
        full_max_wait=40,
        username="tester",
        evolution_report_json=tmp_path / "evolution.json",
        skip_evolution=False,
        skip_rust_regressions=False,
        skip_matrix=False,
        matrix_max_wait=21,
        matrix_max_idle=17,
        chat_id=1,
        chat_b=2,
        chat_c=3,
        user_id=10,
        user_b=11,
        user_c=12,
        session_matrix_report_json=tmp_path / "matrix.json",
        session_matrix_report_markdown=tmp_path / "matrix.md",
        skip_benchmark=False,
        benchmark_iterations=2,
        benchmark_report_json=tmp_path / "benchmark.json",
        project_root=tmp_path,
    )
    env = {"VALKEY_URL": "redis://127.0.0.1:16379/0"}
    memory_suite = tmp_path / "test_omni_agent_memory_suite.py"
    session_matrix = tmp_path / "test_omni_agent_session_matrix.py"
    memory_benchmark = tmp_path / "test_omni_agent_memory_benchmark.py"
    titles: list[str] = []
    gates: list[str] = []

    def _run_command(cmd: list[str], *, title: str, cwd: Path, env: dict[str, str]) -> None:
        assert cmd
        assert str(cwd) == str(tmp_path)
        assert env["VALKEY_URL"] == "redis://127.0.0.1:16379/0"
        titles.append(title)

    def _mark(name: str):
        def _inner(*_args: object, **_kwargs: object) -> None:
            gates.append(name)

        return _inner

    _profiles_module.run_nightly_profile(
        cfg,
        env=env,
        memory_suite=memory_suite,
        session_matrix=session_matrix,
        memory_benchmark=memory_benchmark,
        run_command_fn=_run_command,
        assert_evolution_quality_fn=_mark("evolution"),
        assert_evolution_slow_response_quality_fn=_mark("slow_response"),
        assert_session_matrix_quality_fn=_mark("matrix"),
        assert_benchmark_quality_fn=_mark("benchmark"),
        run_cross_group_complex_gate_fn=_mark("cross_group"),
        run_reflection_quality_gate_fn=_mark("reflection"),
        run_discover_cache_gate_fn=_mark("discover"),
        run_trace_reconstruction_gate_fn=_mark("trace"),
        assert_mcp_waiting_warning_budget_fn=_mark("mcp_wait"),
        assert_memory_stream_warning_budget_fn=_mark("memory_stream"),
    )

    assert titles == [
        "Nightly gate: full memory suite (includes evolution DAG + Rust regressions)",
        "Nightly gate: session matrix",
        "Nightly gate: memory A/B benchmark",
    ]
    assert gates == [
        "evolution",
        "slow_response",
        "matrix",
        "cross_group",
        "benchmark",
        "reflection",
        "discover",
        "trace",
        "mcp_wait",
        "memory_stream",
    ]
