"""Qianji-aligned tests for researcher skill scripts."""

from __future__ import annotations

import asyncio
import importlib
import json
import subprocess
import sys
from pathlib import Path

import pytest

RESEARCHER_SCRIPTS = Path(__file__).parent.parent / "scripts"
if str(RESEARCHER_SCRIPTS) not in sys.path:
    sys.path.insert(0, str(RESEARCHER_SCRIPTS))

research_entry = importlib.import_module("research_entry")
research = importlib.import_module("research")


def _unwrap_skill_output(output: object) -> dict[str, object]:
    """Unwrap @skill_command result payload into the inner JSON object."""
    if isinstance(output, dict):
        content = output.get("content")
        if isinstance(content, list) and content:
            first = content[0]
            if isinstance(first, dict):
                text = first.get("text")
                if isinstance(text, str) and text.strip():
                    parsed = json.loads(text)
                    if isinstance(parsed, dict):
                        return parsed
        return output
    if isinstance(output, str):
        parsed = json.loads(output)
        if isinstance(parsed, dict):
            return parsed
    raise TypeError(f"Unexpected output type: {type(output).__name__}")


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


class TestRunQianjiEngine:
    """Unit tests for Qianji subprocess integration."""

    def test_success_parses_json_marker(self, monkeypatch: pytest.MonkeyPatch) -> None:
        async def _fake_run_subprocess(args: list[str], *, cwd: str, text: bool = True):
            assert "xiuxian-qianji" in " ".join(args)
            assert cwd == "."
            assert text is True
            return _completed_process(
                'boot logs\n=== Final Qianji Execution Result ===\n{"status":"ok","value":1}\n'
            )

        monkeypatch.setattr(research_entry, "_run_subprocess", _fake_run_subprocess)

        success, result, error = asyncio.run(
            research_entry.run_qianji_engine(
                ".",
                {"repo_url": "https://github.com/example/repo.git"},
                "session-1",
            )
        )

        assert success is True
        assert error == ""
        assert result == {"status": "ok", "value": 1}

    def test_nonzero_exit_returns_failure(self, monkeypatch: pytest.MonkeyPatch) -> None:
        async def _fake_run_subprocess(args: list[str], *, cwd: str, text: bool = True):
            return _completed_process("", "qianji failed", 2)

        monkeypatch.setattr(research_entry, "_run_subprocess", _fake_run_subprocess)

        success, result, error = asyncio.run(
            research_entry.run_qianji_engine(".", {"repo_url": "x"}, "session-1")
        )

        assert success is False
        assert result == {}
        assert "qianji failed" in error

    def test_missing_json_marker_returns_failure(self, monkeypatch: pytest.MonkeyPatch) -> None:
        async def _fake_run_subprocess(args: list[str], *, cwd: str, text: bool = True):
            return _completed_process("no marker")

        monkeypatch.setattr(research_entry, "_run_subprocess", _fake_run_subprocess)

        success, result, error = asyncio.run(
            research_entry.run_qianji_engine(".", {"repo_url": "x"}, "session-1")
        )

        assert success is False
        assert result == {}
        assert "Could not find result JSON marker" in error

    def test_invalid_json_after_marker_returns_failure(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        async def _fake_run_subprocess(args: list[str], *, cwd: str, text: bool = True):
            return _completed_process("=== Final Qianji Execution Result ===\n{invalid-json}\n")

        monkeypatch.setattr(research_entry, "_run_subprocess", _fake_run_subprocess)

        success, result, error = asyncio.run(
            research_entry.run_qianji_engine(".", {"repo_url": "x"}, "session-1")
        )

        assert success is False
        assert result == {}
        assert "JSON decode error" in error


@pytest.mark.asyncio
class TestRunResearchGraph:
    """Unit tests for action routing in run_research_graph."""

    async def test_start_filters_non_dict_plan_rows(self, monkeypatch: pytest.MonkeyPatch) -> None:
        async def _fake_run_qianji_engine(project_root: str, context: dict, session_id: str):
            return (
                True,
                {
                    "suspend_prompt": "review shards",
                    "analysis_trace": [
                        {"shard_id": "core", "paths": ["src"]},
                        "drop",
                        1,
                        {"shard_id": "docs", "paths": ["docs"]},
                    ],
                },
                "",
            )

        monkeypatch.setattr(research_entry, "run_qianji_engine", _fake_run_qianji_engine)

        output = await research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            request="Analyze architecture",
            action="start",
        )
        payload = _unwrap_skill_output(output)

        assert payload["success"] is True
        assert payload["message"] == "review shards"
        assert payload["proposed_plan"] == [
            {"shard_id": "core", "paths": ["src"]},
            {"shard_id": "docs", "paths": ["docs"]},
        ]
        assert payload["next_action"].startswith("Call action='approve'")

    async def test_start_propagates_qianji_failure(self, monkeypatch: pytest.MonkeyPatch) -> None:
        async def _fake_run_qianji_engine(project_root: str, context: dict, session_id: str):
            return (False, {}, "engine boom")

        monkeypatch.setattr(research_entry, "run_qianji_engine", _fake_run_qianji_engine)
        output = await research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            action="start",
        )
        payload = _unwrap_skill_output(output)

        assert payload["success"] is False
        assert "engine boom" in str(payload["error"])

    async def test_approve_requires_session_id(self) -> None:
        output = await research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            action="approve",
            session_id="",
            approved_shards='[{"shard_id":"core","paths":["src"]}]',
        )
        payload = _unwrap_skill_output(output)

        assert payload["success"] is False
        assert "session_id" in str(payload["error"]).lower()

    async def test_approve_requires_approved_shards(self) -> None:
        output = await research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            action="approve",
            session_id="sid-1",
            approved_shards="",
        )
        payload = _unwrap_skill_output(output)

        assert payload["success"] is False
        assert "approved_shards" in str(payload["error"]).lower()

    async def test_approve_success_returns_analysis_result(
        self, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        async def _fake_run_qianji_engine(project_root: str, context: dict, session_id: str):
            assert context == {
                "approved_shards": '[{"shard_id":"core","paths":["src"]}]',
            }
            assert session_id == "sid-1"
            return (
                True,
                {"analysis_result": "done", "details": {"count": 2}},
                "",
            )

        monkeypatch.setattr(research_entry, "run_qianji_engine", _fake_run_qianji_engine)
        output = await research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            action="approve",
            session_id="sid-1",
            approved_shards='[{"shard_id":"core","paths":["src"]}]',
        )
        payload = _unwrap_skill_output(output)

        assert payload["success"] is True
        assert payload["analysis_result"] == "done"
        assert payload["full_context"] == {"analysis_result": "done", "details": {"count": 2}}

    async def test_unknown_action_returns_error(self) -> None:
        output = await research_entry.run_research_graph(
            repo_url="https://github.com/example/repo.git",
            action="unknown",
        )
        payload = _unwrap_skill_output(output)

        assert payload["success"] is False
        assert "unknown action" in str(payload["error"]).lower()


class TestResearchUtilities:
    """Unit tests for utility functions in research.py."""

    def test_parse_repo_url_standard_https(self) -> None:
        owner, repo = research.parse_repo_url("https://github.com/anthropics/claude-code")
        assert owner == "anthropics"
        assert repo == "claude-code"

    def test_parse_repo_url_git_suffix(self) -> None:
        owner, repo = research.parse_repo_url("https://github.com/tao3k/omni-dev-fusion.git")
        assert owner == "tao3k"
        assert repo == "omni-dev-fusion"

    def test_parse_repo_url_ssh(self) -> None:
        owner, repo = research.parse_repo_url("git@github.com:antfu/skills.git")
        assert owner == "antfu"
        assert repo == "skills"

    def test_parse_repo_url_raw_githubusercontent(self) -> None:
        owner, repo = research.parse_repo_url(
            "https://raw.githubusercontent.com/user/repo/main/README.md"
        )
        assert owner == "user"
        assert repo == "repo"

    def test_init_harvest_structure_creates_clean_layout(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        monkeypatch.setattr(research, "get_data_dir", lambda name: tmp_path / name)

        stale = tmp_path / "harvested" / "owner" / "repo"
        stale.mkdir(parents=True, exist_ok=True)
        (stale / "old.txt").write_text("old", encoding="utf-8")

        output = research.init_harvest_structure("owner", "repo")
        assert output == tmp_path / "harvested" / "owner" / "repo"
        assert output.exists()
        assert (output / "shards").exists()
        assert not (output / "old.txt").exists()
