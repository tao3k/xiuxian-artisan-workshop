"""Qianji-aligned tests for git smart commit command."""

from __future__ import annotations

import json
import subprocess
import sys
from unittest.mock import AsyncMock

import pytest
from git.scripts.smart_commit_graphflow import commands

from omni.foundation.api.mcp_schema import extract_text_content, parse_result_payload
from omni.foundation.runtime.cargo_subprocess_env import prepare_cargo_subprocess_env


def _unwrap_payload(result: object) -> dict[str, object]:
    """Unwrap decorated skill output into JSON dict payload when possible."""
    parsed = parse_result_payload(result)
    if isinstance(parsed, dict):
        return parsed
    if isinstance(parsed, str):
        return json.loads(parsed)
    raise TypeError(f"Unexpected payload type: {type(parsed).__name__}")


def _completed_process(
    stdout: str,
    stderr: str = "",
    returncode: int = 0,
) -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess(
        args=["cargo", "run"],
        returncode=returncode,
        stdout=stdout,
        stderr=stderr,
    )


def test_prepare_cargo_subprocess_env_rebinds_stale_pyo3_python() -> None:
    env = {
        "PYO3_PYTHON": "/nix/store/does-not-exist-python/bin/python",
        "PYO3_ENVIRONMENT_SIGNATURE": "stale",
        "PYO3_CONFIG_FILE": "/tmp/stale-config",
        "PYO3_NO_PYTHON": "1",
        "PYTHON": sys.executable,
        "DYLD_LIBRARY_PATH": "/tmp/stale-dyld",
    }

    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PYO3_PYTHON"] == sys.executable
    assert prepared["PYO3_ENVIRONMENT_SIGNATURE"] == sys.executable
    assert prepared["DYLD_LIBRARY_PATH"] == "/tmp/stale-dyld"
    assert "PYO3_CONFIG_FILE" not in prepared
    assert "PYO3_NO_PYTHON" not in prepared


def test_prepare_cargo_subprocess_env_keeps_valid_pyo3_python() -> None:
    env = {"PYO3_PYTHON": sys.executable}
    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PYO3_PYTHON"] == sys.executable
    assert prepared["PYO3_ENVIRONMENT_SIGNATURE"] == sys.executable


@pytest.mark.asyncio
class TestSmartCommitWorkflowModular:
    """Unit tests for git smart commit Qianji flow."""

    async def test_start_returns_rendered_result(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(commands, "_handle_submodules_prepare", lambda _project_root: [])
        monkeypatch.setattr(
            commands,
            "run_qianji_engine",
            AsyncMock(
                return_value=(
                    True,
                    {
                        "staged_files": "a.py\nb.py",
                        "security_issues": [],
                        "suspend_prompt": "approve this commit",
                    },
                    "",
                )
            ),
        )
        monkeypatch.setattr(
            commands,
            "_render_start_result",
            lambda *_args, **_kwargs: "Workflow preparation complete",
        )

        out = await commands.smart_commit(action="start", project_root="/tmp/repo")
        text = extract_text_content(out) or ""

        assert "Workflow preparation complete" in text

    async def test_start_returns_engine_error(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(commands, "_handle_submodules_prepare", lambda _project_root: [])
        monkeypatch.setattr(
            commands,
            "run_qianji_engine",
            AsyncMock(return_value=(False, {}, "engine failed")),
        )

        out = await commands.smart_commit(action="start", project_root="/tmp/repo")
        text = extract_text_content(out) or ""

        assert "Workflow Execution Failed" in text
        assert "engine failed" in text

    async def test_approve_requires_workflow_id(self) -> None:
        out = await commands.smart_commit(action="approve", workflow_id="", message="feat(core): x")
        text = extract_text_content(out) or ""
        assert "workflow_id required" in text

    async def test_approve_requires_message(self) -> None:
        out = await commands.smart_commit(action="approve", workflow_id="sid-1", message="")
        text = extract_text_content(out) or ""
        assert "message required" in text

    async def test_approve_scope_validation_error(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(commands, "_get_cog_scopes", lambda _path: ["router", "knowledge"])

        out = await commands.smart_commit(
            action="approve",
            workflow_id="sid-1",
            message="feat(invalid): add validation",
            project_root="/tmp/repo",
        )
        payload = _unwrap_payload(out)

        assert payload["status"] == "error"
        assert "Invalid scope" in str(payload["message"])

    async def test_approve_success_renders_commit(self, monkeypatch: pytest.MonkeyPatch) -> None:
        monkeypatch.setattr(commands, "_get_cog_scopes", lambda _path: [])
        monkeypatch.setattr(
            commands,
            "run_qianji_engine",
            AsyncMock(return_value=(True, {"commit_output": "[main] abc1234 feat: add"}, "")),
        )

        out = await commands.smart_commit(
            action="approve",
            workflow_id="sid-1",
            message="feat(router): add fast path",
            project_root="/tmp/repo",
        )
        text = extract_text_content(out) or ""

        assert "Commit Successful" in text
        assert "sid-1" in text

    async def test_action_normalizes_whitespace_and_case(self) -> None:
        out = await commands.smart_commit(action="  Visualize  ")
        text = extract_text_content(out) or ""

        assert "Qianji Engine" in text
        assert "smart_commit.toml" in text

    async def test_invalid_action_returns_allowed_list(self) -> None:
        out = await commands.smart_commit(action="invalid")
        text = extract_text_content(out) or ""

        assert "action must be one of:" in text
        assert "start" in text
        assert "approve" in text

    async def test_run_qianji_engine_uses_workspace_root_for_cargo(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        captured: dict[str, object] = {}

        async def _fake_run_subprocess(args: list[str], *, cwd, text: bool = True):
            captured["args"] = args
            captured["cwd"] = cwd
            captured["text"] = text
            return _completed_process('=== Final Qianji Execution Result ===\n{"status":"ok"}')

        monkeypatch.setattr(commands, "_run_subprocess", _fake_run_subprocess)
        monkeypatch.setattr(commands, "get_git_toplevel", lambda *_args, **_kwargs: "/workspace")

        ok, payload, err = await commands.run_qianji_engine(
            "/tmp/repo",
            {"project_root": "/tmp/repo"},
            "sid-1",
        )

        assert ok is True
        assert payload == {"status": "ok"}
        assert err == ""
        assert captured["cwd"] == "/workspace"
