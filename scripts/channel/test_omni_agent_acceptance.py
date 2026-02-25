#!/usr/bin/env python3
"""
Unified acceptance runner for Telegram channel black-box validation.

Pipeline:
1) Capture group profile (Test1/Test2/Test3)
2) Command event probes
3) Dedup probe
4) Concurrent session probe
5) Session matrix
6) Complex control-plane scenario
7) Memory evolution DAG scenario
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

DEFAULT_WEBHOOK_LOG = ".run/logs/omni-agent-webhook.log"
DEFAULT_REPORT_JSON = ".run/reports/agent-channel-acceptance.json"
DEFAULT_REPORT_MARKDOWN = ".run/reports/agent-channel-acceptance.md"
DEFAULT_GROUP_PROFILE_JSON = ".run/config/agent-channel-groups.json"
DEFAULT_GROUP_PROFILE_ENV = ".run/config/agent-channel-groups.env"

DEFAULT_MATRIX_JSON = ".run/reports/agent-channel-session-matrix.json"
DEFAULT_MATRIX_MARKDOWN = ".run/reports/agent-channel-session-matrix.md"
DEFAULT_COMPLEX_JSON = ".run/reports/agent-channel-complex-scenarios.json"
DEFAULT_COMPLEX_MARKDOWN = ".run/reports/agent-channel-complex-scenarios.md"
DEFAULT_MEMORY_JSON = ".run/reports/agent-channel-memory-evolution.json"
DEFAULT_MEMORY_MARKDOWN = ".run/reports/agent-channel-memory-evolution.md"

DEFAULT_COMPLEX_DATASET = "scripts/channel/fixtures/complex_blackbox_scenarios.json"
DEFAULT_MEMORY_DATASET = "scripts/channel/fixtures/memory_evolution_complex_scenarios.json"
DEFAULT_MEMORY_SCENARIO = "memory_self_correction_high_complexity_dag"

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_sibling_module = importlib.import_module("module_loader").load_sibling_module

_models_module = load_sibling_module(
    module_name="acceptance_runner_models",
    file_name="acceptance_runner_models.py",
    caller_file=__file__,
    error_context="acceptance runner datamodels",
)
_config_module = load_sibling_module(
    module_name="acceptance_runner_config",
    file_name="acceptance_runner_config.py",
    caller_file=__file__,
    error_context="acceptance runner config helpers",
)
_runtime_module = load_sibling_module(
    module_name="acceptance_runner_runtime",
    file_name="acceptance_runner_runtime.py",
    caller_file=__file__,
    error_context="acceptance runner runtime helpers",
)
_report_module = load_sibling_module(
    module_name="acceptance_runner_report",
    file_name="acceptance_runner_report.py",
    caller_file=__file__,
    error_context="acceptance runner report helpers",
)
_pipeline_module = load_sibling_module(
    module_name="acceptance_runner_pipeline",
    file_name="acceptance_runner_pipeline.py",
    caller_file=__file__,
    error_context="acceptance runner pipeline helpers",
)

StepResult = _models_module.StepResult
AcceptanceConfig = _models_module.AcceptanceConfig


def parse_args() -> argparse.Namespace:
    return _config_module.parse_args(
        default_webhook_log=DEFAULT_WEBHOOK_LOG,
        default_report_json=DEFAULT_REPORT_JSON,
        default_report_markdown=DEFAULT_REPORT_MARKDOWN,
        default_group_profile_json=DEFAULT_GROUP_PROFILE_JSON,
        default_group_profile_env=DEFAULT_GROUP_PROFILE_ENV,
    )


def run_step(
    *,
    step: str,
    title: str,
    cmd: list[str],
    expected_outputs: list[Path],
    attempts: int,
) -> StepResult:
    return _runtime_module.run_step(
        step=step,
        title=title,
        cmd=cmd,
        expected_outputs=expected_outputs,
        attempts=attempts,
        step_result_cls=StepResult,
    )


def build_config(args: argparse.Namespace) -> AcceptanceConfig:
    return _config_module.build_config(args, config_cls=AcceptanceConfig)


def to_markdown(report: dict[str, object]) -> str:
    return _report_module.to_markdown(report)


def write_report(report: dict[str, object], *, output_json: Path, output_markdown: Path) -> None:
    _report_module.write_report(report, output_json=output_json, output_markdown=output_markdown)


def run_pipeline(cfg: AcceptanceConfig) -> dict[str, object]:
    return _pipeline_module.run_pipeline(
        cfg,
        run_step_fn=run_step,
        default_matrix_json=DEFAULT_MATRIX_JSON,
        default_matrix_markdown=DEFAULT_MATRIX_MARKDOWN,
        default_complex_json=DEFAULT_COMPLEX_JSON,
        default_complex_markdown=DEFAULT_COMPLEX_MARKDOWN,
        default_memory_json=DEFAULT_MEMORY_JSON,
        default_memory_markdown=DEFAULT_MEMORY_MARKDOWN,
        default_complex_dataset=DEFAULT_COMPLEX_DATASET,
        default_memory_dataset=DEFAULT_MEMORY_DATASET,
        default_memory_scenario=DEFAULT_MEMORY_SCENARIO,
        python_executable=sys.executable,
    )


def main() -> int:
    cfg = build_config(parse_args())
    report = run_pipeline(cfg)
    write_report(report, output_json=cfg.output_json, output_markdown=cfg.output_markdown)
    print("Acceptance suite completed.", flush=True)
    print(f"  overall={'PASS' if report['overall_passed'] else 'FAIL'}", flush=True)
    print(
        f"  steps={report['summary']['passed']}/{report['summary']['total']}",  # type: ignore[index]
        flush=True,
    )
    print(f"  json_report={cfg.output_json}", flush=True)
    print(f"  markdown_report={cfg.output_markdown}", flush=True)
    return 0 if report["overall_passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
