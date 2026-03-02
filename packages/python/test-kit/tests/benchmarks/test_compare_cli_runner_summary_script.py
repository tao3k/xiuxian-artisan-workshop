"""Unit tests for scripts/compare_cli_runner_summary.py."""

from __future__ import annotations

import json
import subprocess
import sys
from typing import TYPE_CHECKING

from omni.foundation.runtime.gitops import get_project_root

if TYPE_CHECKING:
    from pathlib import Path


def _script_path() -> Path:
    return get_project_root() / "scripts" / "compare_cli_runner_summary.py"


def _artifact_payload(
    *,
    warm_p50: float,
    tool_exec_p50: float,
    bootstrap_p50: float = 12.0,
    mode: str = "default_warm",
) -> dict[str, object]:
    return {
        "schema": "omni.skills.cli_runner_summary.v1",
        "cli_runner_summary": {
            "case_count": 1,
            "profiles": {
                "demo_hello": {
                    "command": "demo.hello",
                    "cases": {
                        mode: {
                            "tool": f"cli.skill_run.{mode}",
                            "p50_ms": warm_p50,
                            "timing_breakdown_ms": {
                                "bootstrap": {"p50_ms": bootstrap_p50},
                                "daemon_connect": {"p50_ms": 45.0},
                                "tool_exec": {"p50_ms": tool_exec_p50},
                            },
                            "ok": True,
                        }
                    },
                    "comparisons": {},
                }
            },
            "rank_by_p50_ms": [],
        },
    }


def _run_compare(
    base_file: Path, target_file: Path, *args: str
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(_script_path()), str(base_file), str(target_file), *args],
        check=False,
        capture_output=True,
        text=True,
    )


def test_compare_cli_runner_summary_passes_without_regression(tmp_path: Path) -> None:
    base_file = tmp_path / "base.json"
    target_file = tmp_path / "target.json"
    base_file.write_text(
        json.dumps(_artifact_payload(warm_p50=200.0, tool_exec_p50=120.0), ensure_ascii=False),
        encoding="utf-8",
    )
    target_file.write_text(
        json.dumps(_artifact_payload(warm_p50=180.0, tool_exec_p50=100.0), ensure_ascii=False),
        encoding="utf-8",
    )

    result = _run_compare(base_file, target_file, "--fail-on-regression")
    assert result.returncode == 0
    payload = json.loads(result.stdout)
    assert payload["regression_count"] == 0
    assert payload["missing_metric_count"] == 0


def test_compare_cli_runner_summary_fails_on_regression(tmp_path: Path) -> None:
    base_file = tmp_path / "base.json"
    target_file = tmp_path / "target.json"
    base_file.write_text(
        json.dumps(_artifact_payload(warm_p50=200.0, tool_exec_p50=120.0), ensure_ascii=False),
        encoding="utf-8",
    )
    target_file.write_text(
        json.dumps(_artifact_payload(warm_p50=260.0, tool_exec_p50=180.0), ensure_ascii=False),
        encoding="utf-8",
    )

    result = _run_compare(
        base_file,
        target_file,
        "--fail-on-regression",
        "--max-regression-ms",
        "20",
        "--max-regression-ratio",
        "1.05",
    )
    assert result.returncode == 1
    payload = json.loads(result.stdout)
    assert payload["regression_count"] >= 1
    regressions = [item for item in payload["comparisons"] if item.get("regression") is True]
    assert regressions


def test_compare_cli_runner_summary_bootstrap_override_reduces_noise(tmp_path: Path) -> None:
    base_file = tmp_path / "base.json"
    target_file = tmp_path / "target.json"
    base_file.write_text(
        json.dumps(
            _artifact_payload(
                warm_p50=200.0,
                tool_exec_p50=120.0,
                bootstrap_p50=20.0,
            ),
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )
    target_file.write_text(
        json.dumps(
            _artifact_payload(
                warm_p50=200.0,
                tool_exec_p50=120.0,
                bootstrap_p50=62.0,
            ),
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )

    default_result = _run_compare(
        base_file,
        target_file,
        "--fail-on-regression",
        "--max-regression-ms",
        "20",
        "--max-regression-ratio",
        "1.05",
    )
    assert default_result.returncode == 1

    override_result = _run_compare(
        base_file,
        target_file,
        "--fail-on-regression",
        "--max-regression-ms",
        "20",
        "--max-regression-ratio",
        "1.05",
        "--bootstrap-max-regression-ms",
        "60",
        "--bootstrap-max-regression-ratio",
        "3.2",
    )
    assert override_result.returncode == 0
    payload = json.loads(override_result.stdout)
    assert payload["regression_count"] == 0
    assert payload["thresholds"]["bootstrap_max_regression_ms"] == 60.0
    assert payload["thresholds"]["bootstrap_max_regression_ratio"] == 3.2


def test_compare_cli_runner_summary_skips_default_cold_bootstrap_metric(tmp_path: Path) -> None:
    base_file = tmp_path / "base.json"
    target_file = tmp_path / "target.json"
    base_file.write_text(
        json.dumps(
            _artifact_payload(
                warm_p50=900.0,
                tool_exec_p50=200.0,
                bootstrap_p50=80.0,
                mode="default_cold",
            ),
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )
    target_file.write_text(
        json.dumps(
            _artifact_payload(
                warm_p50=900.0,
                tool_exec_p50=200.0,
                bootstrap_p50=260.0,
                mode="default_cold",
            ),
            ensure_ascii=False,
        ),
        encoding="utf-8",
    )

    result = _run_compare(
        base_file,
        target_file,
        "--fail-on-regression",
        "--max-regression-ms",
        "20",
        "--max-regression-ratio",
        "1.05",
        "--bootstrap-max-regression-ms",
        "20",
        "--bootstrap-max-regression-ratio",
        "1.05",
    )
    assert result.returncode == 0
    payload = json.loads(result.stdout)
    assert payload["regression_count"] == 0
    assert all(
        item["metric"] != "timing.bootstrap.p50_ms" for item in payload.get("comparisons", [])
    )
