#!/usr/bin/env python3
"""Unit tests for memory CI gate runner helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import SimpleNamespace

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

_runner_module = importlib.import_module("memory_ci_gate_runner")
run_gate = _runner_module.run_gate


class _Handle:
    def __init__(self) -> None:
        self.closed = False

    def close(self) -> None:
        self.closed = True


class _Process:
    pass


def _write_script(path: Path) -> None:
    path.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    path.chmod(0o755)


def test_run_gate_quick_profile_invokes_expected_steps(tmp_path: Path) -> None:
    script_dir = tmp_path / "scripts"
    script_dir.mkdir(parents=True, exist_ok=True)
    for name in (
        "valkey-start.sh",
        "valkey-stop.sh",
        "mock_telegram_api.py",
        "test_omni_agent_memory_suite.py",
        "test_omni_agent_session_matrix.py",
        "test_omni_agent_memory_benchmark.py",
    ):
        _write_script(script_dir / name)

    report_root = tmp_path / ".run" / "reports"
    cfg = SimpleNamespace(
        profile="quick",
        project_root=tmp_path,
        script_dir=script_dir,
        agent_bin=tmp_path / "target" / "debug" / "omni-agent",
        webhook_port=19090,
        telegram_api_port=19091,
        valkey_port=16379,
        valkey_url="redis://127.0.0.1:16379/0",
        valkey_prefix="omni-agent:test",
        username="tester",
        webhook_secret="secret",
        chat_id=1,
        chat_b=2,
        chat_c=3,
        user_id=10,
        user_b=11,
        user_c=12,
        runtime_log_file=report_root / "runtime.log",
        mock_log_file=report_root / "mock.log",
        evolution_report_json=report_root / "evolution.json",
        benchmark_report_json=report_root / "benchmark.json",
        session_matrix_report_json=report_root / "matrix.json",
        session_matrix_report_markdown=report_root / "matrix.md",
        trace_report_json=report_root / "trace.json",
        trace_report_markdown=report_root / "trace.md",
        cross_group_report_json=report_root / "cross.json",
        cross_group_report_markdown=report_root / "cross.md",
        runtime_startup_timeout_secs=10,
        quick_max_wait=25,
        quick_max_idle=20,
        skip_rust_regressions=True,
    )

    invoked_titles: list[str] = []
    invoked_gates: list[str] = []
    terminated: list[str] = []

    def _run_command(cmd: list[str], *, title: str, cwd: Path, env: dict[str, str]) -> None:
        assert cmd
        assert str(cwd) == str(tmp_path)
        assert env["VALKEY_URL"] == cfg.valkey_url
        invoked_titles.append(title)

    def _start_background_process(
        cmd: list[str], *, cwd: Path, env: dict[str, str], log_file: Path, title: str
    ) -> tuple[_Process, _Handle]:
        assert cmd
        assert str(cwd) == str(tmp_path)
        log_file.parent.mkdir(parents=True, exist_ok=True)
        return _Process(), _Handle()

    def _mark(name: str):
        def _inner(*args: object, **kwargs: object) -> None:
            del args, kwargs
            invoked_gates.append(name)

        return _inner

    def _terminate(process: object | None, *, name: str) -> None:
        del process
        terminated.append(name)

    run_gate(
        cfg,
        ensure_parent_dirs_fn=lambda *paths: [
            path.parent.mkdir(parents=True, exist_ok=True) for path in paths
        ],
        default_run_suffix_fn=lambda: "testrun",
        write_ci_channel_acl_settings_fn=lambda _cfg, config_home: config_home / "settings.yaml",
        valkey_reachable_fn=lambda _url: False,
        run_command_fn=_run_command,
        start_background_process_fn=_start_background_process,
        wait_for_mock_health_fn=lambda *_args, **_kwargs: None,
        wait_for_log_regex_fn=lambda *_args, **_kwargs: None,
        run_reflection_quality_gate_fn=_mark("reflection"),
        run_discover_cache_gate_fn=_mark("discover"),
        run_trace_reconstruction_gate_fn=_mark("trace"),
        run_cross_group_complex_gate_fn=_mark("cross"),
        assert_mcp_waiting_warning_budget_fn=_mark("mcp_wait"),
        assert_memory_stream_warning_budget_fn=_mark("memory_stream"),
        assert_evolution_quality_fn=_mark("evolution"),
        assert_evolution_slow_response_quality_fn=_mark("slow_response"),
        assert_session_matrix_quality_fn=_mark("matrix"),
        assert_benchmark_quality_fn=_mark("benchmark"),
        terminate_process_fn=_terminate,
    )

    assert "Start Valkey" in invoked_titles
    assert any("Quick gate: memory suite" in title for title in invoked_titles)
    assert invoked_gates == ["reflection", "discover", "trace", "mcp_wait", "memory_stream"]
    assert terminated == ["omni-agent runtime", "mock Telegram API"]
