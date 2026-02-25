#!/usr/bin/env python3
"""
MCP startup regression suite (hot + cold) for omni-agent gateway startup.
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

load_sibling_module = importlib.import_module("module_loader").load_sibling_module

_path_module = load_sibling_module(
    module_name="path_resolver",
    file_name="path_resolver.py",
    caller_file=__file__,
    error_context="shared path resolver",
)
_models_module = load_sibling_module(
    module_name="mcp_startup_suite_models",
    file_name="mcp_startup_suite_models.py",
    caller_file=__file__,
    error_context="MCP startup suite datamodels",
)
_config_module = load_sibling_module(
    module_name="mcp_startup_suite_config",
    file_name="mcp_startup_suite_config.py",
    caller_file=__file__,
    error_context="MCP startup suite config helpers",
)
_runtime_module = load_sibling_module(
    module_name="mcp_startup_suite_runtime",
    file_name="mcp_startup_suite_runtime.py",
    caller_file=__file__,
    error_context="MCP startup suite runtime helpers",
)
_quality_module = load_sibling_module(
    module_name="mcp_startup_suite_quality",
    file_name="mcp_startup_suite_quality.py",
    caller_file=__file__,
    error_context="MCP startup suite quality helpers",
)
_report_module = load_sibling_module(
    module_name="mcp_startup_suite_report",
    file_name="mcp_startup_suite_report.py",
    caller_file=__file__,
    error_context="MCP startup suite report helpers",
)

SuiteConfig = _models_module.SuiteConfig
ModeSpec = _models_module.ModeSpec


def default_report_path(filename: str) -> Path:
    return _path_module.default_report_path(filename)


def project_root_from(start: Path) -> Path:
    return _path_module.project_root_from(start)


def resolve_path(path_str: str, project_root: Path) -> Path:
    return _path_module.resolve_path(path_str, project_root)


def parse_args() -> argparse.Namespace:
    return _config_module.parse_args()


def build_config(args: argparse.Namespace) -> SuiteConfig:
    return _config_module.build_config(args, config_cls=SuiteConfig)


def shell_join(parts: list[str]) -> str:
    return _runtime_module.shell_join(parts)


def build_restart_command(cfg: SuiteConfig) -> str:
    return _runtime_module.build_restart_command(cfg)


def build_mode_specs(cfg: SuiteConfig) -> tuple[ModeSpec, ...]:
    return _runtime_module.build_mode_specs(cfg, mode_spec_cls=ModeSpec)


def mode_report_paths(cfg: SuiteConfig, mode: str) -> tuple[Path, Path]:
    return _runtime_module.mode_report_paths(cfg, mode)


def load_summary(path: Path) -> dict[str, object] | None:
    return _runtime_module.load_summary(path)


def run_shell_command(command: str, cwd: Path) -> tuple[int, str]:
    return _runtime_module.run_shell_command(command, cwd)


def run_mode(cfg: SuiteConfig, spec: ModeSpec) -> dict[str, object]:
    return _runtime_module.run_mode(cfg, spec)


def _mode_p95(summary: dict[str, object]) -> float:
    return _quality_module.mode_p95(summary)


def _mode_failed(summary: dict[str, object]) -> int:
    return _quality_module.mode_failed(summary)


def _load_baseline_mode_p95s(path: Path) -> dict[str, float]:
    return _quality_module.load_baseline_mode_p95s(path)


def evaluate_quality_gates(cfg: SuiteConfig, modes: list[dict[str, object]]) -> dict[str, object]:
    return _quality_module.evaluate_quality_gates(cfg, modes)


def render_markdown(report: dict[str, object]) -> str:
    return _report_module.render_markdown(report)


def run_suite(cfg: SuiteConfig) -> dict[str, object]:
    return _runtime_module.run_suite(
        cfg,
        build_mode_specs_fn=build_mode_specs,
        evaluate_quality_gates_fn=evaluate_quality_gates,
    )


def write_report(report: dict[str, object], output_json: Path, output_markdown: Path) -> None:
    _report_module.write_report(report, output_json, output_markdown)


def main() -> int:
    try:
        cfg = build_config(parse_args())
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    report = run_suite(cfg)
    write_report(report, cfg.output_json, cfg.output_markdown)

    print("MCP startup suite completed.", flush=True)
    print(
        (
            f"  modes={len(report['modes'])} "
            f"passed={report['passed_modes']} failed={report['failed_modes']}"
        ),
        flush=True,
    )
    quality_gate = report.get("quality_gate", {})
    if isinstance(quality_gate, dict):
        print(f"  quality_gate_passed={quality_gate.get('passed', True)}", flush=True)
        violations = quality_gate.get("violations")
        if isinstance(violations, list) and violations:
            print("  quality_violations:", flush=True)
            for violation in violations:
                print(f"    - {violation}", flush=True)
    print(f"  json_report={cfg.output_json}", flush=True)
    print(f"  markdown_report={cfg.output_markdown}", flush=True)
    return 0 if report["overall_passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
