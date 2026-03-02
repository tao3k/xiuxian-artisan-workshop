"""Tests for minimal JSON-only CLI runner path."""

from __future__ import annotations

import asyncio
import io
import json
import sys
from typing import Any


def test_run_skills_json_unwraps_canonical_payload(monkeypatch) -> None:
    """JSON fast runner should print pure business payload."""
    from omni.agent.cli.runner_json import run_skills_json

    async def _fake_run_tool_with_monitor(
        _tool: str,
        _args: dict[str, Any],
        *,
        output_json: bool = False,
        auto_report: bool = False,
    ):
        assert output_json is True
        assert auto_report is False
        return (
            {
                "content": [{"type": "text", "text": '{"status":"success","found":3}'}],
                "isError": False,
            },
            None,
        )

    async def _fake_close_embedding_client() -> None:
        return None

    def _run_async_blocking(awaitable):
        return asyncio.run(awaitable)

    monkeypatch.setattr("omni.agent.cli.runner_json.run_async_blocking", _run_async_blocking)
    monkeypatch.setattr(
        "omni.core.skills.runner.run_tool_with_monitor", _fake_run_tool_with_monitor
    )
    monkeypatch.setattr(
        "omni.foundation.api.tool_context.run_with_execution_timeout",
        lambda coro: coro,
    )
    monkeypatch.setattr(
        "omni.foundation.embedding_client.close_embedding_client",
        _fake_close_embedding_client,
    )

    stdout_capture = io.StringIO()
    with monkeypatch.context() as m:
        m.setattr(sys, "stdout", stdout_capture)
        code = run_skills_json(["knowledge.search", '{"query":"x"}'])

    assert code == 0
    out = stdout_capture.getvalue()
    assert '"status": "success"' in out
    assert '"found": 3' in out
    assert '"content"' not in out
    assert '"isError"' not in out


def test_run_skills_json_reports_invalid_json_args(monkeypatch) -> None:
    """Invalid JSON args should return non-zero and structured error payload."""
    from omni.agent.cli.runner_json import run_skills_json

    stdout_capture = io.StringIO()
    with monkeypatch.context() as m:
        m.setattr(sys, "stdout", stdout_capture)
        code = run_skills_json(["knowledge.search", '{"query":'])

    assert code == 1
    out = stdout_capture.getvalue()
    assert '"success": false' in out
    assert '"status": "error"' in out
    assert "Invalid JSON args" in out


def test_run_skills_json_reuse_process_delegates_to_daemon(monkeypatch) -> None:
    """Reuse-process mode should emit daemon response payload."""
    from omni.agent.cli import runner_json as runner_json_module

    def _fake_daemon(commands, *, quiet=True):
        assert quiet is True
        assert commands == ["knowledge.search", '{"query":"x"}']
        return 0, '{"status":"success","source":"daemon"}'

    monkeypatch.setattr(runner_json_module, "_run_skills_json_via_daemon", _fake_daemon)

    stdout_capture = io.StringIO()
    with monkeypatch.context() as m:
        m.setattr(sys, "stdout", stdout_capture)
        code = runner_json_module.run_skills_json(
            ["knowledge.search", '{"query":"x"}'],
            reuse_process=True,
        )

    assert code == 0
    out = stdout_capture.getvalue()
    assert '"status":"success"' in out
    assert '"source":"daemon"' in out


def test_get_runner_daemon_status_reports_running(monkeypatch) -> None:
    """Status helper should parse daemon ping response."""
    from omni.agent.cli import runner_json as runner_json_module

    monkeypatch.setattr(
        runner_json_module,
        "_request_runner_daemon_ping",
        lambda _socket_path: (
            0,
            '{"success":true,"status":"ok","daemon":"skill-runner-json","pid":42}',
        ),
    )

    status = runner_json_module.get_runner_daemon_status()
    assert status["running"] is True
    assert status["pid"] == 42


def test_stop_runner_daemon_returns_stopped(monkeypatch) -> None:
    """Stop helper should surface stopped=True on successful shutdown response."""
    from omni.agent.cli import runner_json as runner_json_module

    monkeypatch.setattr(
        runner_json_module,
        "_request_runner_daemon_shutdown",
        lambda _socket_path: (0, '{"success":true,"status":"stopping"}'),
    )

    status = runner_json_module.stop_runner_daemon()
    assert status["stopped"] is True


def test_run_skills_json_emits_timing_to_stderr_when_enabled(monkeypatch) -> None:
    """Timing env should emit one prefixed JSON payload on stderr."""
    from omni.agent.cli import runner_json as runner_json_module

    async def _fake_run_tool_with_monitor(
        _tool: str,
        _args: dict[str, Any],
        *,
        output_json: bool = False,
        auto_report: bool = False,
    ):
        assert output_json is True
        assert auto_report is False
        return ({"status": "success", "found": 1}, None)

    async def _fake_close_embedding_client() -> None:
        return None

    def _run_async_blocking(awaitable):
        return asyncio.run(awaitable)

    monkeypatch.setattr(runner_json_module, "run_async_blocking", _run_async_blocking)
    monkeypatch.setattr(
        "omni.core.skills.runner.run_tool_with_monitor",
        _fake_run_tool_with_monitor,
    )
    monkeypatch.setattr(
        "omni.foundation.api.tool_context.run_with_execution_timeout",
        lambda coro: coro,
    )
    monkeypatch.setattr(
        "omni.foundation.embedding_client.close_embedding_client",
        _fake_close_embedding_client,
    )
    monkeypatch.setenv("OMNI_SKILL_RUN_TIMING", "1")

    stdout_capture = io.StringIO()
    stderr_capture = io.StringIO()
    with monkeypatch.context() as m:
        m.setattr(sys, "stdout", stdout_capture)
        m.setattr(sys, "stderr", stderr_capture)
        code = runner_json_module.run_skills_json(["knowledge.search", '{"query":"x"}'])

    assert code == 0
    stderr_lines = [line for line in stderr_capture.getvalue().splitlines() if line.strip()]
    assert len(stderr_lines) == 1
    assert stderr_lines[0].startswith(runner_json_module._TIMING_PREFIX)

    timing_payload = json.loads(stderr_lines[0][len(runner_json_module._TIMING_PREFIX) :])
    assert timing_payload["mode"] == "local"
    assert isinstance(timing_payload["bootstrap_ms"], float)
    assert isinstance(timing_payload["tool_exec_ms"], float)


def test_get_daemon_request_timeout_seconds_prefers_env(monkeypatch) -> None:
    """Daemon request timeout should prefer explicit env override."""
    from omni.agent.cli import runner_json as runner_json_module

    monkeypatch.setenv("OMNI_SKILL_RUNNER_REQUEST_TIMEOUT", "45")
    assert runner_json_module.get_daemon_request_timeout_seconds() == 45.0


def test_get_daemon_request_timeout_seconds_uses_mcp_timeout(monkeypatch) -> None:
    """Daemon request timeout should fall back to settings mcp.timeout."""
    from omni.agent.cli import runner_json as runner_json_module

    monkeypatch.delenv("OMNI_SKILL_RUNNER_REQUEST_TIMEOUT", raising=False)
    monkeypatch.setattr(
        "omni.foundation.config.settings.get_setting",
        lambda key, default=None: 120 if key == "mcp.timeout" else default,
    )
    assert runner_json_module.get_daemon_request_timeout_seconds() == 120.0
