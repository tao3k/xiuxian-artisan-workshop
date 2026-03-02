import asyncio
import importlib
import json
import subprocess
import sys
from pathlib import Path

from omni.foundation.runtime.cargo_subprocess_env import prepare_cargo_subprocess_env

RESEARCHER_SCRIPTS = Path(__file__).parent.parent / "scripts"
if str(RESEARCHER_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(RESEARCHER_SCRIPTS))

research_entry = importlib.import_module("research_entry")


def _completed_process(
    stdout: str, stderr: str = "", returncode: int = 0
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
    env = {
        "PYO3_PYTHON": sys.executable,
    }

    prepared = prepare_cargo_subprocess_env(env)

    assert prepared["PYO3_PYTHON"] == sys.executable
    assert prepared["PYO3_ENVIRONMENT_SIGNATURE"] == sys.executable


def test_run_qianji_engine_uses_default_subprocess_env(monkeypatch) -> None:
    calls: dict[str, object] = {}

    async def _fake_run_subprocess(args, *, cwd, extra_env=None, text=True):
        calls["args"] = args
        calls["cwd"] = cwd
        calls["extra_env"] = extra_env
        calls["text"] = text
        return _completed_process(
            'boot logs\n=== Final Qianji Execution Result ===\n{"status": "ok"}\n'
        )

    monkeypatch.setattr(research_entry, "_run_subprocess", _fake_run_subprocess)

    success, result, error = asyncio.run(
        research_entry.run_qianji_engine(
            ".",
            {"repo_url": "https://example.com/repo.git"},
            "session-1",
        )
    )

    assert success is True
    assert error == ""
    assert result["status"] == "ok"
    assert calls["cwd"] == "."
    assert calls["extra_env"] is None
    assert calls["text"] is True


def test_run_qianji_engine_reports_missing_json_marker(monkeypatch) -> None:
    async def _fake_run_subprocess(args, *, cwd, extra_env=None, text=True):
        return _completed_process("no result marker found")

    monkeypatch.setattr(research_entry, "_run_subprocess", _fake_run_subprocess)

    success, result, error = asyncio.run(
        research_entry.run_qianji_engine(".", {"repo_url": "x"}, "session-1")
    )

    assert success is False
    assert result == {}
    assert "Could not find result JSON marker" in error


def test_run_research_graph_start_filters_non_dict_analysis_entries(monkeypatch) -> None:
    async def _fake_run_qianji_engine(project_root, context, session_id):
        return (
            True,
            {
                "suspend_prompt": "review shards",
                "analysis_trace": [
                    {"shard_id": "core", "paths": ["src"]},
                    "ignore-me",
                    7,
                    {"shard_id": "docs", "paths": ["docs"]},
                ],
            },
            "",
        )

    monkeypatch.setattr(research_entry, "run_qianji_engine", _fake_run_qianji_engine)

    output = asyncio.run(
        research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            request="Analyze architecture",
            action="start",
        )
    )

    if isinstance(output, dict) and "content" in output:
        payload = json.loads(output["content"][0]["text"])
    else:
        payload = output

    assert payload["success"] is True
    assert payload["message"] == "review shards"
    assert payload["proposed_plan"] == [
        {"shard_id": "core", "paths": ["src"]},
        {"shard_id": "docs", "paths": ["docs"]},
    ]
