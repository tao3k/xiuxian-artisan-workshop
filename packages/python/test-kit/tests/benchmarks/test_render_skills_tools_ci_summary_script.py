"""Unit tests for scripts/render_skills_tools_ci_summary.py."""

from __future__ import annotations

import json
import subprocess
import sys
from typing import TYPE_CHECKING

from omni.foundation.runtime.gitops import get_project_root

if TYPE_CHECKING:
    from pathlib import Path


def _script_path() -> Path:
    return get_project_root() / "scripts" / "render_skills_tools_ci_summary.py"


def _write_json(path: Path, payload: dict[str, object]) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def _run_script(
    *,
    deterministic_report: Path,
    cli_diff_report: Path,
    remote_fetch_report: Path,
    network_report: Path,
    baseline_file: Path,
    artifact_file: Path,
    previous_status_json: Path | None,
    output_json: Path,
    output_markdown: Path,
    strict: bool = False,
    max_overall_regression_streak: int = 0,
    max_component_regression_streak: int = 0,
    strict_trend_alert: bool = False,
) -> subprocess.CompletedProcess[str]:
    cmd = [
        sys.executable,
        str(_script_path()),
        "--deterministic-report",
        str(deterministic_report),
        "--cli-diff-report",
        str(cli_diff_report),
        "--remote-fetch-report",
        str(remote_fetch_report),
        "--network-report",
        str(network_report),
        "--baseline-file",
        str(baseline_file),
        "--artifact-file",
        str(artifact_file),
        "--previous-status-json",
        str(previous_status_json) if previous_status_json is not None else "",
        "--output-json",
        str(output_json),
        "--output-markdown",
        str(output_markdown),
        "--max-overall-regression-streak",
        str(max_overall_regression_streak),
        "--max-component-regression-streak",
        str(max_component_regression_streak),
    ]
    if strict:
        cmd.append("--strict")
    if strict_trend_alert:
        cmd.append("--strict-trend-alert")
    return subprocess.run(cmd, check=False, capture_output=True, text=True)


def test_render_summary_reports_ok_status(tmp_path: Path) -> None:
    deterministic = tmp_path / "deterministic_gate.json"
    diff = tmp_path / "cli_runner_summary_diff.json"
    remote = tmp_path / "cli_runner_summary_remote_fetch.json"
    network = tmp_path / "crawl4ai_network_observability.json"
    baseline = tmp_path / "cli_runner_summary.base.json"
    artifact = tmp_path / "cli_runner_summary.json"
    previous_status = tmp_path / "skills_tools_ci_status.previous.json"
    output_json = tmp_path / "skills_tools_ci_status.json"
    output_md = tmp_path / "skills_tools_ci_status.md"

    _write_json(
        deterministic,
        {"errors": [], "snapshot": {"anomaly_count": 0}, "cli_runner_gate": {"violation_count": 0}},
    )
    _write_json(diff, {"regression_count": 0, "missing_metric_count": 0})
    _write_json(
        remote, {"status": "ok", "run_id": 123, "member_name": "cli_runner_summary.base.json"}
    )
    _write_json(network, {"success": True, "status": "ok"})
    _write_json(
        previous_status,
        {
            "overall_status": "warn",
            "components": {
                "remote_fetch": {"status": "skipped"},
                "deterministic_gate": {"status": "error"},
                "cli_diff_gate": {"status": "ok"},
                "baseline": {"status": "missing"},
                "network_observability": {"status": "warn"},
            },
        },
    )
    baseline.write_text("{}", encoding="utf-8")
    artifact.write_text("{}", encoding="utf-8")

    result = _run_script(
        deterministic_report=deterministic,
        cli_diff_report=diff,
        remote_fetch_report=remote,
        network_report=network,
        baseline_file=baseline,
        artifact_file=artifact,
        previous_status_json=previous_status,
        output_json=output_json,
        output_markdown=output_md,
    )

    assert result.returncode == 0
    status_payload = json.loads(output_json.read_text(encoding="utf-8"))
    assert status_payload["overall_status"] == "ok"
    assert status_payload["trend"]["overall"]["change"] == "improved"
    assert status_payload["trend"]["components"]["deterministic_gate"]["change"] == "improved"
    assert status_payload["components"]["remote_fetch"]["status"] == "ok"
    markdown = output_md.read_text(encoding="utf-8")
    assert "| `remote_fetch` | `ok` |" in markdown


def test_render_summary_reports_warn_for_skipped_and_network_warn(tmp_path: Path) -> None:
    deterministic = tmp_path / "deterministic_gate.json"
    diff = tmp_path / "cli_runner_summary_diff.json"
    remote = tmp_path / "cli_runner_summary_remote_fetch.json"
    network = tmp_path / "crawl4ai_network_observability.json"
    baseline = tmp_path / "cli_runner_summary.base.json"
    artifact = tmp_path / "cli_runner_summary.json"
    output_json = tmp_path / "skills_tools_ci_status.json"
    output_md = tmp_path / "skills_tools_ci_status.md"

    _write_json(
        deterministic,
        {"errors": [], "snapshot": {"anomaly_count": 0}, "cli_runner_gate": {"violation_count": 0}},
    )
    _write_json(diff, {"status": "skipped", "reason": "baseline_missing"})
    _write_json(remote, {"status": "skipped", "reason": "artifact_or_member_not_found"})
    _write_json(
        network, {"success": False, "status": "network_observability_failed", "exit_code": 7}
    )
    artifact.write_text("{}", encoding="utf-8")

    result = _run_script(
        deterministic_report=deterministic,
        cli_diff_report=diff,
        remote_fetch_report=remote,
        network_report=network,
        baseline_file=baseline,
        artifact_file=artifact,
        previous_status_json=None,
        output_json=output_json,
        output_markdown=output_md,
    )

    assert result.returncode == 0
    status_payload = json.loads(output_json.read_text(encoding="utf-8"))
    assert status_payload["overall_status"] == "warn"
    assert status_payload["trend"]["overall"]["change"] == "unknown"
    assert status_payload["components"]["remote_fetch"]["status"] == "skipped"
    assert status_payload["components"]["network_observability"]["status"] == "warn"


def test_render_summary_strict_fails_on_error(tmp_path: Path) -> None:
    deterministic = tmp_path / "deterministic_gate.json"
    diff = tmp_path / "cli_runner_summary_diff.json"
    remote = tmp_path / "cli_runner_summary_remote_fetch.json"
    network = tmp_path / "crawl4ai_network_observability.json"
    baseline = tmp_path / "cli_runner_summary.base.json"
    artifact = tmp_path / "cli_runner_summary.json"
    previous_status = tmp_path / "skills_tools_ci_status.previous.json"
    output_json = tmp_path / "skills_tools_ci_status.json"
    output_md = tmp_path / "skills_tools_ci_status.md"

    _write_json(
        deterministic,
        {
            "errors": ["x"],
            "snapshot": {"anomaly_count": 1},
            "cli_runner_gate": {"violation_count": 0},
        },
    )
    _write_json(diff, {"regression_count": 0, "missing_metric_count": 0})
    _write_json(
        remote, {"status": "ok", "run_id": 123, "member_name": "cli_runner_summary.base.json"}
    )
    _write_json(network, {"success": True, "status": "ok"})
    _write_json(previous_status, {"overall_status": "ok", "components": {}})
    baseline.write_text("{}", encoding="utf-8")
    artifact.write_text("{}", encoding="utf-8")

    result = _run_script(
        deterministic_report=deterministic,
        cli_diff_report=diff,
        remote_fetch_report=remote,
        network_report=network,
        baseline_file=baseline,
        artifact_file=artifact,
        previous_status_json=previous_status,
        output_json=output_json,
        output_markdown=output_md,
        strict=True,
    )

    assert result.returncode == 1


def test_render_summary_trend_alert_records_violations(tmp_path: Path) -> None:
    deterministic = tmp_path / "deterministic_gate.json"
    diff = tmp_path / "cli_runner_summary_diff.json"
    remote = tmp_path / "cli_runner_summary_remote_fetch.json"
    network = tmp_path / "crawl4ai_network_observability.json"
    baseline = tmp_path / "cli_runner_summary.base.json"
    artifact = tmp_path / "cli_runner_summary.json"
    previous_status = tmp_path / "skills_tools_ci_status.previous.json"
    output_json = tmp_path / "skills_tools_ci_status.json"
    output_md = tmp_path / "skills_tools_ci_status.md"

    _write_json(
        deterministic,
        {"errors": [], "snapshot": {"anomaly_count": 0}, "cli_runner_gate": {"violation_count": 0}},
    )
    _write_json(diff, {"regression_count": 0, "missing_metric_count": 0})
    _write_json(
        remote, {"status": "ok", "run_id": 123, "member_name": "cli_runner_summary.base.json"}
    )
    _write_json(
        network,
        {"success": False, "status": "network_observability_failed", "exit_code": 7},
    )
    _write_json(
        previous_status,
        {
            "overall_status": "warn",
            "components": {
                "network_observability": {"status": "warn"},
            },
            "trend": {
                "overall": {"regression_streak": 1},
                "components": {
                    "network_observability": {"regression_streak": 1},
                },
            },
        },
    )
    baseline.write_text("{}", encoding="utf-8")
    artifact.write_text("{}", encoding="utf-8")

    result = _run_script(
        deterministic_report=deterministic,
        cli_diff_report=diff,
        remote_fetch_report=remote,
        network_report=network,
        baseline_file=baseline,
        artifact_file=artifact,
        previous_status_json=previous_status,
        output_json=output_json,
        output_markdown=output_md,
        max_overall_regression_streak=2,
        max_component_regression_streak=2,
    )

    assert result.returncode == 0
    status_payload = json.loads(output_json.read_text(encoding="utf-8"))
    trend_alert = status_payload["trend_alert"]
    assert trend_alert["enabled"] is True
    assert trend_alert["violation_count"] == 2
    assert trend_alert["violations"][0]["scope"] == "overall"
    assert trend_alert["violations"][1]["scope"] == "component"
    assert trend_alert["violations"][1]["component"] == "network_observability"


def test_render_summary_strict_trend_alert_fails_on_violation(tmp_path: Path) -> None:
    deterministic = tmp_path / "deterministic_gate.json"
    diff = tmp_path / "cli_runner_summary_diff.json"
    remote = tmp_path / "cli_runner_summary_remote_fetch.json"
    network = tmp_path / "crawl4ai_network_observability.json"
    baseline = tmp_path / "cli_runner_summary.base.json"
    artifact = tmp_path / "cli_runner_summary.json"
    previous_status = tmp_path / "skills_tools_ci_status.previous.json"
    output_json = tmp_path / "skills_tools_ci_status.json"
    output_md = tmp_path / "skills_tools_ci_status.md"

    _write_json(
        deterministic,
        {"errors": [], "snapshot": {"anomaly_count": 0}, "cli_runner_gate": {"violation_count": 0}},
    )
    _write_json(diff, {"regression_count": 0, "missing_metric_count": 0})
    _write_json(
        remote, {"status": "ok", "run_id": 123, "member_name": "cli_runner_summary.base.json"}
    )
    _write_json(
        network,
        {"success": False, "status": "network_observability_failed", "exit_code": 7},
    )
    _write_json(
        previous_status,
        {
            "overall_status": "warn",
            "components": {
                "network_observability": {"status": "warn"},
            },
            "trend": {
                "overall": {"regression_streak": 1},
                "components": {
                    "network_observability": {"regression_streak": 1},
                },
            },
        },
    )
    baseline.write_text("{}", encoding="utf-8")
    artifact.write_text("{}", encoding="utf-8")

    result = _run_script(
        deterministic_report=deterministic,
        cli_diff_report=diff,
        remote_fetch_report=remote,
        network_report=network,
        baseline_file=baseline,
        artifact_file=artifact,
        previous_status_json=previous_status,
        output_json=output_json,
        output_markdown=output_md,
        max_overall_regression_streak=2,
        max_component_regression_streak=2,
        strict_trend_alert=True,
    )

    assert result.returncode == 1


def test_render_summary_trend_streak_increments_on_persistent_error(tmp_path: Path) -> None:
    deterministic = tmp_path / "deterministic_gate.json"
    diff = tmp_path / "cli_runner_summary_diff.json"
    remote = tmp_path / "cli_runner_summary_remote_fetch.json"
    network = tmp_path / "crawl4ai_network_observability.json"
    baseline = tmp_path / "cli_runner_summary.base.json"
    artifact = tmp_path / "cli_runner_summary.json"
    previous_status = tmp_path / "skills_tools_ci_status.previous.json"
    output_json = tmp_path / "skills_tools_ci_status.json"
    output_md = tmp_path / "skills_tools_ci_status.md"

    _write_json(
        deterministic,
        {"errors": [], "snapshot": {"anomaly_count": 0}, "cli_runner_gate": {"violation_count": 0}},
    )
    _write_json(diff, {"regression_count": 1, "missing_metric_count": 0})
    _write_json(
        remote, {"status": "ok", "run_id": 123, "member_name": "cli_runner_summary.base.json"}
    )
    _write_json(network, {"success": True, "status": "ok"})
    _write_json(
        previous_status,
        {
            "overall_status": "error",
            "components": {"cli_diff_gate": {"status": "error"}},
            "trend": {
                "overall": {"regression_streak": 1},
                "components": {"cli_diff_gate": {"regression_streak": 1}},
            },
        },
    )
    baseline.write_text("{}", encoding="utf-8")
    artifact.write_text("{}", encoding="utf-8")

    result = _run_script(
        deterministic_report=deterministic,
        cli_diff_report=diff,
        remote_fetch_report=remote,
        network_report=network,
        baseline_file=baseline,
        artifact_file=artifact,
        previous_status_json=previous_status,
        output_json=output_json,
        output_markdown=output_md,
        max_overall_regression_streak=2,
        max_component_regression_streak=2,
        strict_trend_alert=True,
    )

    assert result.returncode == 1
    status_payload = json.loads(output_json.read_text(encoding="utf-8"))
    assert status_payload["trend"]["overall"]["change"] == "unchanged"
    assert status_payload["trend"]["overall"]["regression_streak"] == 2
    assert status_payload["trend"]["components"]["cli_diff_gate"]["regression_streak"] == 2
