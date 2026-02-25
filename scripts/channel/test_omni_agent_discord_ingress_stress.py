#!/usr/bin/env python3
"""
Discord ingress stress probe for omni-agent.

This runner posts concurrent synthetic Discord ingress events and measures:
- HTTP success/failure ratio
- latency + RPS
- queue-pressure signals from runtime logs

Outputs:
- JSON report
- Markdown report
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
    module_name="discord_ingress_stress_models",
    file_name="discord_ingress_stress_models.py",
    caller_file=__file__,
    error_context="discord ingress stress datamodels",
)
_config_module = load_sibling_module(
    module_name="discord_ingress_stress_config",
    file_name="discord_ingress_stress_config.py",
    caller_file=__file__,
    error_context="discord ingress stress config helpers",
)
_runtime_module = load_sibling_module(
    module_name="discord_ingress_stress_runtime",
    file_name="discord_ingress_stress_runtime.py",
    caller_file=__file__,
    error_context="discord ingress stress runtime helpers",
)
_report_module = load_sibling_module(
    module_name="discord_ingress_stress_report",
    file_name="discord_ingress_stress_report.py",
    caller_file=__file__,
    error_context="discord ingress stress report helpers",
)

StressConfig = _models_module.StressConfig
RoundResult = _models_module.RoundResult


def parse_args() -> argparse.Namespace:
    """Parse CLI arguments."""
    return _config_module.parse_args()


def build_config(args: argparse.Namespace) -> StressConfig:
    """Build validated stress config."""
    return _config_module.build_config(args, config_cls=StressConfig)


def run_stress(cfg: StressConfig) -> dict[str, object]:
    """Execute stress run and return report object."""
    return _runtime_module.run_stress(cfg, round_result_cls=RoundResult)


def write_report(report: dict[str, object], output_json: Path, output_markdown: Path) -> None:
    """Write JSON + Markdown reports."""
    _report_module.write_report(report, output_json, output_markdown)


def main() -> int:
    """CLI entrypoint."""
    try:
        cfg = build_config(parse_args())
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    report = run_stress(cfg)
    write_report(report, cfg.output_json, cfg.output_markdown)

    summary = report["summary"]
    print("Discord ingress stress completed.")
    print(
        "  requests={total} success={ok} failed={failed}".format(
            total=summary["total_requests"],
            ok=summary["success_requests"],
            failed=summary["failed_requests"],
        )
    )
    print(
        "  failure_rate={:.2%} max_round_p95_ms={:.2f} avg_rps={:.2f}".format(
            float(summary["failure_rate"]),
            float(summary["max_round_p95_ms"]),
            float(summary["average_rps"]),
        )
    )
    print(
        "  queue_wait={} gate_wait={} queue_unavailable={}".format(
            summary["queue_wait_events"],
            summary["foreground_gate_wait_events"],
            summary["inbound_queue_unavailable_events"],
        )
    )
    print(f"  json_report={cfg.output_json}")
    print(f"  markdown_report={cfg.output_markdown}")

    return 0 if bool(summary["quality_passed"]) else 1


if __name__ == "__main__":
    raise SystemExit(main())
