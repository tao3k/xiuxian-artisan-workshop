"""Unit tests for scripts/benchmark_skills_tools_ci.sh."""

from __future__ import annotations

import os
import subprocess
from typing import TYPE_CHECKING

from omni.foundation.runtime.gitops import get_project_root

if TYPE_CHECKING:
    from pathlib import Path


def _ci_script_path() -> Path:
    return get_project_root() / "scripts" / "benchmark_skills_tools_ci.sh"


def _run_ci_dry(
    *args: str,
    env_overrides: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    script = _ci_script_path()
    env = dict(os.environ)
    env["OMNI_SKILLS_TOOLS_CI_DRY_RUN"] = "1"
    env["OMNI_SKILLS_TOOLS_CLI_SUMMARY_PROMOTE_BASELINE"] = "1"
    if env_overrides:
        env.update(env_overrides)
    return subprocess.run(
        ["bash", str(script), *args],
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )


def test_ci_script_dry_run_defaults() -> None:
    result = _run_ci_dry()
    assert result.returncode == 0
    out = result.stdout
    assert "mkdir -p .run/reports/skills-tools-benchmark" in out
    assert "benchmark_skills_tools_gate.sh deterministic 3" in out
    assert "--cli-summary-file .run/reports/skills-tools-benchmark/cli_runner_summary.json" in out
    assert "# baseline: .run/reports/skills-tools-benchmark/cli_runner_summary.base.json" in out
    assert "# diff_report: .run/reports/skills-tools-benchmark/cli_runner_summary_diff.json" in out
    assert (
        "# remote_fetch_report: .run/reports/skills-tools-benchmark/cli_runner_summary_remote_fetch.json"
        in out
    )
    assert (
        "# previous_status_file: .run/reports/skills-tools-benchmark/skills_tools_ci_status.previous.json"
        in out
    )
    assert (
        "# previous_status_fetch_report: "
        ".run/reports/skills-tools-benchmark/skills_tools_ci_status_previous_fetch.json" in out
    )
    assert (
        "# ci_status_json: .run/reports/skills-tools-benchmark/skills_tools_ci_status.json" in out
    )
    assert (
        "# ci_status_markdown: .run/reports/skills-tools-benchmark/skills_tools_ci_status.md" in out
    )
    assert "# trend_thresholds: overall=0 component=0 strict=0" in out
    assert (
        "# cli_diff_thresholds: max_ms=70 max_ratio=1.2 bootstrap_max_ms=60 bootstrap_max_ratio=1.7"
        in out
    )
    assert "render_skills_tools_ci_summary.py" in out
    assert "--max-overall-regression-streak 0" in out
    assert "--max-component-regression-streak 0" in out
    assert "--strict-trend-alert" not in out
    assert (
        "cp -f .run/reports/skills-tools-benchmark/cli_runner_summary.json .run/reports/skills-tools-benchmark/cli_runner_summary.base.json"
        in out
    )
    assert "benchmark_skills_tools_gate.sh network 5" in out
    assert "deterministic_gate.json" in out
    assert "cli_runner_summary.json" in out
    assert "cli_runner_summary_diff.json" in out
    assert "crawl4ai_network_observability.json" in out


def test_ci_script_dry_run_honors_custom_args() -> None:
    result = _run_ci_dry("/tmp/custom-report", "7", "9")
    assert result.returncode == 0
    out = result.stdout
    assert "mkdir -p /tmp/custom-report" in out
    assert "benchmark_skills_tools_gate.sh deterministic 7" in out
    assert "--cli-summary-file /tmp/custom-report/cli_runner_summary.json" in out
    assert "# baseline: /tmp/custom-report/cli_runner_summary.base.json" in out
    assert "# diff_report: /tmp/custom-report/cli_runner_summary_diff.json" in out
    assert "# remote_fetch_report: /tmp/custom-report/cli_runner_summary_remote_fetch.json" in out
    assert "# previous_status_file: /tmp/custom-report/skills_tools_ci_status.previous.json" in out
    assert (
        "# previous_status_fetch_report: "
        "/tmp/custom-report/skills_tools_ci_status_previous_fetch.json" in out
    )
    assert "# ci_status_json: /tmp/custom-report/skills_tools_ci_status.json" in out
    assert "# ci_status_markdown: /tmp/custom-report/skills_tools_ci_status.md" in out
    assert "# trend_thresholds: overall=0 component=0 strict=0" in out
    assert (
        "# cli_diff_thresholds: max_ms=70 max_ratio=1.2 bootstrap_max_ms=60 bootstrap_max_ratio=1.7"
        in out
    )
    assert "render_skills_tools_ci_summary.py" in out
    assert "--max-overall-regression-streak 0" in out
    assert "--max-component-regression-streak 0" in out
    assert "--strict-trend-alert" not in out
    assert (
        "cp -f /tmp/custom-report/cli_runner_summary.json /tmp/custom-report/cli_runner_summary.base.json"
        in out
    )
    assert "benchmark_skills_tools_gate.sh network 9" in out


def test_ci_script_dry_run_honors_explicit_baseline_arg() -> None:
    result = _run_ci_dry("/tmp/custom-report", "7", "9", "/tmp/base_summary.json")
    assert result.returncode == 0
    out = result.stdout
    assert "# baseline: /tmp/base_summary.json" in out


def test_ci_script_dry_run_prints_remote_fetch_command_when_configured(tmp_path: Path) -> None:
    script = _ci_script_path()
    env = dict(os.environ)
    env["OMNI_SKILLS_TOOLS_CI_DRY_RUN"] = "1"
    env["OMNI_SKILLS_TOOLS_CLI_SUMMARY_PROMOTE_BASELINE"] = "1"
    env["OMNI_SKILLS_TOOLS_REMOTE_ARTIFACT_NAME"] = "skills-tools-benchmark-ubuntu-latest"
    result = subprocess.run(
        ["bash", str(script), str(tmp_path)],
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )
    assert result.returncode == 0
    out = result.stdout
    assert "fetch_previous_skills_benchmark_artifact.py" in out
    assert "--artifact-name skills-tools-benchmark-ubuntu-latest" in out
    assert "--preferred-member skills_tools_ci_status.json" in out


def test_ci_script_dry_run_can_disable_baseline_promotion() -> None:
    script = _ci_script_path()
    env = dict(os.environ)
    env["OMNI_SKILLS_TOOLS_CI_DRY_RUN"] = "1"
    env["OMNI_SKILLS_TOOLS_CLI_SUMMARY_PROMOTE_BASELINE"] = "0"
    result = subprocess.run(
        ["bash", str(script)],
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )
    assert result.returncode == 0
    out = result.stdout
    assert "baseline promotion disabled" in out
    assert "cp -f " not in out


def test_ci_script_dry_run_honors_trend_alert_env() -> None:
    result = _run_ci_dry(
        env_overrides={
            "OMNI_SKILLS_TOOLS_TREND_MAX_OVERALL_STREAK": "3",
            "OMNI_SKILLS_TOOLS_TREND_MAX_COMPONENT_STREAK": "4",
            "OMNI_SKILLS_TOOLS_TREND_STRICT": "1",
        }
    )
    assert result.returncode == 0
    out = result.stdout
    assert "# trend_thresholds: overall=3 component=4 strict=1" in out
    assert "--max-overall-regression-streak 3" in out
    assert "--max-component-regression-streak 4" in out
    assert "--strict-trend-alert" in out


def test_ci_script_dry_run_honors_bootstrap_diff_threshold_env() -> None:
    result = _run_ci_dry(
        env_overrides={
            "OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_MS": "60",
            "OMNI_SKILLS_TOOLS_CLI_SUMMARY_DIFF_BOOTSTRAP_MAX_RATIO": "1.7",
        }
    )
    assert result.returncode == 0
    out = result.stdout
    assert (
        "# cli_diff_thresholds: max_ms=70 max_ratio=1.2 bootstrap_max_ms=60 bootstrap_max_ratio=1.7"
        in out
    )
    assert "--bootstrap-max-regression-ms 60" in out
    assert "--bootstrap-max-regression-ratio 1.7" in out
