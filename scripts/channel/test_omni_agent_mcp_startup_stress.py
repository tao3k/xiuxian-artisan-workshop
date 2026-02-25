#!/usr/bin/env python3
"""
MCP startup stress probe for omni-agent.

This probe repeatedly starts `omni-agent gateway` processes and verifies MCP
handshake robustness under reconnect pressure.

Outputs:
  - JSON report (machine-readable)
  - Markdown report (human-readable)
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse
    from collections.abc import Iterable

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
    module_name="mcp_startup_stress_models",
    file_name="mcp_startup_stress_models.py",
    caller_file=__file__,
    error_context="MCP startup stress datamodels",
)
_config_module = load_sibling_module(
    module_name="mcp_startup_stress_config",
    file_name="mcp_startup_stress_config.py",
    caller_file=__file__,
    error_context="MCP startup stress config helpers",
)
_runtime_module = load_sibling_module(
    module_name="mcp_startup_stress_runtime",
    file_name="mcp_startup_stress_runtime.py",
    caller_file=__file__,
    error_context="MCP startup stress runtime helpers",
)
_report_module = load_sibling_module(
    module_name="mcp_startup_stress_report",
    file_name="mcp_startup_stress_report.py",
    caller_file=__file__,
    error_context="MCP startup stress report helpers",
)

StressConfig = _models_module.StressConfig
ProbeResult = _models_module.ProbeResult
HealthSample = _models_module.HealthSample


def parse_args() -> argparse.Namespace:
    return _config_module.parse_args()


def default_report_path(filename: str) -> Path:
    return _path_module.default_report_path(filename)


def project_root_from(start: Path) -> Path:
    return _path_module.project_root_from(start)


def resolve_path(path_str: str, project_root: Path) -> Path:
    return _path_module.resolve_path(path_str, project_root)


def build_config(args: argparse.Namespace) -> StressConfig:
    return _config_module.build_config(args, config_cls=StressConfig)


def check_health(url: str, timeout_secs: float = 2.0) -> tuple[bool, str]:
    return _runtime_module.check_health(url, timeout_secs)


def run_restart_command(command: str, cwd: Path) -> tuple[int, str]:
    return _runtime_module.run_restart_command(command, cwd)


def classify_reason(
    *,
    ready_seen: bool,
    handshake_timeout_seen: bool,
    connect_failed_seen: bool,
    process_exited: bool,
    timed_out: bool,
) -> str:
    return _runtime_module.classify_reason(
        ready_seen=ready_seen,
        handshake_timeout_seen=handshake_timeout_seen,
        connect_failed_seen=connect_failed_seen,
        process_exited=process_exited,
        timed_out=timed_out,
    )


def p95(values: list[float]) -> float:
    return _runtime_module.p95(values)


def summarize_health_samples(samples: Iterable[HealthSample]) -> dict[str, object]:
    return _runtime_module.summarize_health_samples(samples)


def _collect_health_sample(url: str, timeout_secs: float) -> HealthSample:
    return _runtime_module.collect_health_sample(
        url,
        timeout_secs,
        health_sample_cls=HealthSample,
    )


def run_single_probe(cfg: StressConfig, round_index: int, worker_index: int) -> ProbeResult:
    return _runtime_module.run_single_probe(
        cfg,
        round_index,
        worker_index,
        probe_result_cls=ProbeResult,
    )


def summarize(
    results: Iterable[ProbeResult],
    health_samples: Iterable[HealthSample],
) -> dict[str, object]:
    return _runtime_module.summarize(results, health_samples)


def render_markdown(report: dict[str, object]) -> str:
    return _report_module.render_markdown(report)


def run_stress(cfg: StressConfig) -> dict[str, object]:
    return _runtime_module.run_stress(
        cfg,
        probe_result_cls=ProbeResult,
        health_sample_cls=HealthSample,
    )


def write_report(report: dict[str, object], output_json: Path, output_markdown: Path) -> None:
    _report_module.write_report(report, output_json, output_markdown)


def main() -> int:
    try:
        cfg = build_config(parse_args())
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    try:
        report = run_stress(cfg)
    except RuntimeError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    write_report(report, cfg.output_json, cfg.output_markdown)
    summary = report["summary"]
    print("MCP startup stress completed.")
    print(
        "  probes={total} passed={passed} failed={failed}".format(
            total=summary["total"], passed=summary["passed"], failed=summary["failed"]
        )
    )
    print(
        "  success_avg_startup_ms={:.1f} success_p95_startup_ms={:.1f}".format(
            summary["success_avg_startup_ms"], summary["success_p95_startup_ms"]
        )
    )
    print(
        "  health_failure_rate={:.2%} health_p95_latency_ms={:.1f}".format(
            summary["health_failure_rate"],
            summary["health_p95_latency_ms"],
        )
    )
    print(f"  json_report={cfg.output_json}")
    print(f"  markdown_report={cfg.output_markdown}")

    return 0 if summary["failed"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
