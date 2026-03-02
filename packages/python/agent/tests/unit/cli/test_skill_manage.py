"""
test_skill_manage.py - Skill Management Commands Tests

Tests for:
- run: Execute a skill command
- test: Test skills using the testing framework
- check: Validate skill structure
- install: Install a skill from a remote repository [DEPRECATED]
- update: Update an installed skill [DEPRECATED]

Usage:
    uv run pytest packages/python/agent/tests/unit/cli/test_skill_manage.py -v
"""

from __future__ import annotations

from typing import TYPE_CHECKING
from unittest.mock import patch

import pytest
from typer.testing import CliRunner

from omni.agent.cli.app import app
from omni.agent.cli.commands.skill.manage import _extract_skill_name_from_nodeid

if TYPE_CHECKING:
    from pathlib import Path


class TestSkillRun:
    """Tests for 'omni skill run' command."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_run_requires_command(self, runner):
        """Test that run requires a command argument."""
        result = runner.invoke(app, ["skill", "run"])

        assert result.exit_code != 0

    def test_run_with_command_format(self, runner):
        """Test run with skill.command format."""
        result = runner.invoke(app, ["skill", "run", "nonexistent.command"])

        # Should not be a usage error
        assert "requires" not in result.output.lower()

    def test_run_with_reuse_process_forwards_to_runner(self, runner, monkeypatch):
        """`skill run --reuse-process` should call run_skills with reuse flag."""
        calls: dict[str, object] = {}

        def _fake_run_skills(
            commands,
            *,
            json_output=False,
            quiet=True,
            log_handler=None,
            reuse_process=False,
        ):
            calls["commands"] = list(commands)
            calls["json_output"] = bool(json_output)
            calls["reuse_process"] = bool(reuse_process)

        monkeypatch.setattr(
            "omni.agent.cli.commands.skill.manage.run_skills",
            _fake_run_skills,
        )

        result = runner.invoke(app, ["skill", "run", "knowledge.search", "--reuse-process"])

        assert result.exit_code == 0
        assert calls["commands"] == ["knowledge.search"]
        assert calls["json_output"] is False
        assert calls["reuse_process"] is True

    def test_run_defaults_to_reuse_process(self, runner, monkeypatch):
        """`skill run` should use daemon reuse by default."""
        calls: dict[str, object] = {}

        def _fake_run_skills(
            commands,
            *,
            json_output=False,
            quiet=True,
            log_handler=None,
            reuse_process=False,
        ):
            calls["commands"] = list(commands)
            calls["json_output"] = bool(json_output)
            calls["reuse_process"] = bool(reuse_process)

        monkeypatch.setattr(
            "omni.agent.cli.commands.skill.manage.run_skills",
            _fake_run_skills,
        )

        result = runner.invoke(app, ["skill", "run", "knowledge.search"])

        assert result.exit_code == 0
        assert calls["commands"] == ["knowledge.search"]
        assert calls["json_output"] is False
        assert calls["reuse_process"] is True

    def test_run_no_reuse_process_disables_daemon(self, runner, monkeypatch):
        """`skill run --no-reuse-process` should disable daemon reuse."""
        calls: dict[str, object] = {}

        def _fake_run_skills(
            commands,
            *,
            json_output=False,
            quiet=True,
            log_handler=None,
            reuse_process=False,
        ):
            calls["commands"] = list(commands)
            calls["json_output"] = bool(json_output)
            calls["reuse_process"] = bool(reuse_process)

        monkeypatch.setattr(
            "omni.agent.cli.commands.skill.manage.run_skills",
            _fake_run_skills,
        )

        result = runner.invoke(app, ["skill", "run", "knowledge.search", "--no-reuse-process"])

        assert result.exit_code == 0
        assert calls["commands"] == ["knowledge.search"]
        assert calls["json_output"] is False
        assert calls["reuse_process"] is False


class TestSkillTest:
    """Tests for 'omni skill test' command."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_test_no_args_shows_usage(self, runner, tmp_path: Path):
        """Test test without arguments shows usage."""
        skills_dir = tmp_path / "assets" / "skills"
        skills_dir.mkdir(parents=True)

        with patch("omni.foundation.config.skills.SKILLS_DIR", return_value=skills_dir):
            result = runner.invoke(app, ["skill", "test"])

        assert result.exit_code == 0
        assert "Specify" in result.output or "usage" in result.output.lower()

    def test_test_missing_skill(self, runner, tmp_path: Path):
        """Test test with non-existent skill."""
        skills_dir = tmp_path / "assets" / "skills"
        skills_dir.mkdir(parents=True)

        with patch("omni.foundation.config.skills.SKILLS_DIR", return_value=skills_dir):
            result = runner.invoke(app, ["skill", "test", "nonexistent"])

        assert result.exit_code == 1
        assert "not found" in result.output.lower()

    def test_test_skill_without_tests(self, runner, tmp_path: Path):
        """Test test with skill that has no tests directory."""
        skills_dir = tmp_path / "assets" / "skills"
        skills_dir.mkdir(parents=True)

        skill_dir = skills_dir / "no_tests"
        skill_dir.mkdir()
        (skill_dir / "SKILL.md").write_text("""
---
name: no_tests
version: 1.0.0
---
""")

        with patch("omni.foundation.config.skills.SKILLS_DIR", return_value=skills_dir):
            result = runner.invoke(app, ["skill", "test", "no_tests"])

        assert result.exit_code == 1
        assert "No tests" in result.output or "not found" in result.output


class TestSkillCheck:
    """Tests for 'omni skill check' command."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_check_valid_skill(self, runner, tmp_path: Path):
        """Test check with a valid skill structure."""
        skills_dir = tmp_path / "assets" / "skills"
        skills_dir.mkdir(parents=True)

        skill_dir = skills_dir / "valid_skill"
        skill_dir.mkdir()
        (skill_dir / "SKILL.md").write_text("""
---
name: valid_skill
version: 1.0.0
description: A valid skill
---
""")

        with patch("omni.foundation.config.skills.SKILLS_DIR", return_value=skills_dir):
            result = runner.invoke(app, ["skill", "check", "valid_skill"])

        assert result.exit_code == 0
        assert "Valid" in result.output or "valid_skill" in result.output

    def test_check_missing_skill(self, runner, tmp_path: Path):
        """Test check with non-existent skill."""
        skills_dir = tmp_path / "assets" / "skills"
        skills_dir.mkdir(parents=True)

        with patch("omni.foundation.config.skills.SKILLS_DIR", return_value=skills_dir):
            result = runner.invoke(app, ["skill", "check", "nonexistent"])

        assert (
            "not found" in result.output.lower()
            or "Invalid" in result.output
            or result.exit_code == 0
        )

    def test_check_all_skills(self, runner, tmp_path: Path):
        """Test check with all skills."""
        skills_dir = tmp_path / "assets" / "skills"
        skills_dir.mkdir(parents=True)

        for skill_name in ["skill1", "skill2"]:
            skill_dir = skills_dir / skill_name
            skill_dir.mkdir()
            (skill_dir / "SKILL.md").write_text(f"""
---
name: {skill_name}
version: 1.0.0
description: A skill
---
""")

        with patch("omni.foundation.config.skills.SKILLS_DIR", return_value=skills_dir):
            result = runner.invoke(app, ["skill", "check"])

        assert result.exit_code == 0
        assert "Skill Structure Check" in result.output


class TestSkillNodeIdParsing:
    """Tests for robust skill extraction from pytest nodeid paths."""

    def test_extract_skill_name_from_project_relative_nodeid(self):
        nodeid = "assets/skills/knowledge/tests/test_recall_filter.py::test_recall_single_call"
        assert _extract_skill_name_from_nodeid(nodeid, {"knowledge", "code"}) == "knowledge"

    def test_extract_skill_name_from_absolute_nodeid(self):
        nodeid = (
            "/Users/dev/repo/assets/skills/advanced_tools/tests/"
            "test_advanced_tools_modular.py::TestAdvancedToolsModular::test_smart_search"
        )
        assert (
            _extract_skill_name_from_nodeid(nodeid, {"advanced_tools", "memory"})
            == "advanced_tools"
        )

    def test_extract_skill_name_from_windows_nodeid(self):
        nodeid = (
            r"C:\repo\assets\skills\crawl4ai\tests\test_graph.py::"
            r"TestExtractSkeleton::test_extract_stats"
        )
        assert _extract_skill_name_from_nodeid(nodeid, {"crawl4ai", "writer"}) == "crawl4ai"

    def test_extract_skill_name_returns_none_when_unmatched(self):
        nodeid = "packages/python/agent/tests/unit/test_misc.py::test_smoke"
        assert _extract_skill_name_from_nodeid(nodeid, {"knowledge", "code"}) is None


class TestSkillInstallUnavailable:
    """Tests for 'omni skill install' availability messaging."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_install_shows_unavailable(self, runner):
        """Test that install command shows unavailable message."""
        result = runner.invoke(app, ["skill", "install", "https://github.com/example/skill"])

        assert result.exit_code == 0
        assert "Unavailable" in result.output or "not available" in result.output.lower()


class TestSkillUpdateUnavailable:
    """Tests for 'omni skill update' availability messaging."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_update_shows_unavailable(self, runner):
        """Test that update command shows unavailable message."""
        result = runner.invoke(app, ["skill", "update", "some_skill"])

        assert result.exit_code == 0
        assert "Unavailable" in result.output or "not available" in result.output.lower()


class TestSkillRunnerDaemon:
    """Tests for `omni skill runner` daemon commands."""

    @pytest.fixture
    def runner(self):
        return CliRunner()

    def test_runner_status_command(self, runner, monkeypatch):
        """runner status should display daemon state."""
        monkeypatch.setattr(
            "omni.agent.cli.runner_json.get_runner_daemon_status",
            lambda: {"running": True, "pid": 123},
        )
        result = runner.invoke(app, ["skill", "runner", "status"])
        assert result.exit_code == 0
        assert "running" in result.output.lower()

    def test_runner_start_command(self, runner, monkeypatch):
        """runner start should call start helper and print success."""
        monkeypatch.setattr(
            "omni.agent.cli.runner_json.start_runner_daemon",
            lambda: {"running": True, "started": True, "pid": 321},
        )
        result = runner.invoke(app, ["skill", "runner", "start"])
        assert result.exit_code == 0
        assert "started" in result.output.lower() or "running" in result.output.lower()

    def test_runner_stop_command(self, runner, monkeypatch):
        """runner stop should call stop helper and print success."""
        monkeypatch.setattr(
            "omni.agent.cli.runner_json.stop_runner_daemon",
            lambda: {"stopped": True},
        )
        result = runner.invoke(app, ["skill", "runner", "stop"])
        assert result.exit_code == 0
        assert "stopped" in result.output.lower()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
