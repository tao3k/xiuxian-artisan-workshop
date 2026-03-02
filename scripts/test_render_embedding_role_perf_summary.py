from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path

import pytest

_MODULE_PATH = Path(__file__).resolve().with_name("render_embedding_role_perf_summary.py")
_SPEC = importlib.util.spec_from_file_location("render_embedding_role_perf_summary", _MODULE_PATH)
assert _SPEC and _SPEC.loader
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)

render_summary = _MODULE.render_summary


def _sample_payload(status: str = "pass") -> dict[str, object]:
    return {
        "schema": "omni_agent.embedding.role_perf_smoke.v1",
        "status": status,
        "duration_secs": 12.34,
        "upstream_base_url": "http://127.0.0.1:11434",
        "embedding_model": "qwen3-embedding:0.6b",
        "single_runs": 30,
        "batch_runs": 12,
        "concurrent_total": 64,
        "concurrent_width": 8,
        "roles": [
            {
                "role": "litellm_rs",
                "single": {"p95_ms": 88.1, "err": 0},
                "batch8": {"p95_ms": 210.0, "err": 0},
                "concurrent_single": {"rps": 54.2, "p95_ms": 220.3, "err": 0},
            },
            {
                "role": "mistral_sdk",
                "single": {"p95_ms": 84.2, "err": 1},
                "batch8": {"p95_ms": 205.0, "err": 0},
                "concurrent_single": {"rps": 56.8, "p95_ms": 200.1, "err": 0},
            },
        ],
        "failures": [],
    }


def test_render_summary_includes_key_metrics_table() -> None:
    markdown = render_summary(
        _sample_payload(), profile="medium", runner_os="ubuntu-latest", report_path="/tmp/r.json"
    )

    assert "## Embedding Role Perf (medium, ubuntu-latest)" in markdown
    assert "| Role | single p95 (ms) | batch8 p95 (ms) | concurrent RPS |" in markdown
    assert "| litellm_rs | 88.10 | 210.00 | 54.20 | 220.30 | 0 |" in markdown
    assert "| mistral_sdk | 84.20 | 205.00 | 56.80 | 200.10 | 1 |" in markdown
    assert "- Status: `pass`" in markdown
    assert "- Report: `/tmp/r.json`" in markdown


def test_render_summary_includes_failures_section_when_present() -> None:
    payload = _sample_payload(status="fail")
    payload["failures"] = ["litellm_rs: concurrent rps too low"]
    markdown = render_summary(payload, profile="heavy")

    assert "- Status: `fail`" in markdown
    assert "### Failures" in markdown
    assert "- litellm_rs: concurrent rps too low" in markdown


def test_render_summary_rejects_schema_mismatch() -> None:
    payload = _sample_payload()
    payload["schema"] = "wrong.schema"
    with pytest.raises(ValueError, match="schema mismatch"):
        render_summary(payload)


def test_cli_writes_markdown_file(tmp_path: Path) -> None:
    report_path = tmp_path / "report.json"
    output_md = tmp_path / "summary.md"
    report_path.write_text(json.dumps(_sample_payload()) + "\n", encoding="utf-8")

    result = subprocess.run(
        [
            sys.executable,
            str(_MODULE_PATH),
            "--input",
            str(report_path),
            "--output-markdown",
            str(output_md),
            "--profile",
            "medium",
            "--runner-os",
            "ubuntu-latest",
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0
    assert "Embedding Role Perf" in result.stdout
    assert output_md.exists()
    assert "litellm_rs" in output_md.read_text(encoding="utf-8")
