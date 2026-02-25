#!/usr/bin/env python3
"""
Aggregate omni-agent memory/session black-box reports into a single SLO gate.
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

_models_module = load_sibling_module(
    module_name="memory_slo_models",
    file_name="memory_slo_models.py",
    caller_file=__file__,
    error_context="memory slo datamodels",
)
_config_module = load_sibling_module(
    module_name="memory_slo_config",
    file_name="memory_slo_config.py",
    caller_file=__file__,
    error_context="memory slo config helpers",
)
_runtime_module = load_sibling_module(
    module_name="memory_slo_runtime",
    file_name="memory_slo_runtime.py",
    caller_file=__file__,
    error_context="memory slo runtime helpers",
)
_output_module = load_sibling_module(
    module_name="memory_slo_output",
    file_name="memory_slo_output.py",
    caller_file=__file__,
    error_context="memory slo output helpers",
)

SloConfig = _models_module.SloConfig

default_report_path = _config_module.default_report_path
project_root_from = _config_module.project_root_from
resolve_path = _config_module.resolve_path
parse_required_modes = _config_module.parse_required_modes
load_json = _runtime_module.load_json
evaluate_evolution = _runtime_module.evaluate_evolution
evaluate_benchmark = _runtime_module.evaluate_benchmark
evaluate_session_matrix = _runtime_module.evaluate_session_matrix
evaluate_stream_health = _runtime_module.evaluate_stream_health
render_markdown = _output_module.render_markdown
write_outputs = _output_module.write_outputs


def parse_args() -> argparse.Namespace:
    return _config_module.parse_args()


def build_config(args: argparse.Namespace) -> SloConfig:
    return _config_module.build_config(args, config_cls=SloConfig)


def run_slo_report(cfg: SloConfig) -> dict[str, object]:
    return _runtime_module.run_slo_report(cfg)


def main() -> int:
    try:
        cfg = build_config(parse_args())
        report = run_slo_report(cfg)
        write_outputs(report, cfg.output_json, cfg.output_markdown)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2
    except RuntimeError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    print("Memory SLO aggregation completed.", flush=True)
    print(f"  overall={'PASS' if report['overall_passed'] else 'FAIL'}", flush=True)
    print(f"  failure_count={report['failure_count']}", flush=True)
    print(f"  json_report={cfg.output_json}", flush=True)
    print(f"  markdown_report={cfg.output_markdown}", flush=True)
    if report["failures"]:
        print("  failures:", flush=True)
        for item in report["failures"]:
            print(f"    - {item}", flush=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
