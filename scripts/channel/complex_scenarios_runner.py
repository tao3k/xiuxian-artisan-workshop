#!/usr/bin/env python3
"""Main orchestration for complex scenario suite execution."""

from __future__ import annotations

import sys
import time
from typing import Any


def run_complex_scenarios_main(
    *,
    parse_args_fn: Any,
    build_config_fn: Any,
    load_scenarios_fn: Any,
    select_scenarios_fn: Any,
    run_scenario_fn: Any,
    build_report_fn: Any,
    write_outputs_fn: Any,
    datetime_cls: Any,
    utc_tz: Any,
) -> int:
    """Run scenario suite, write reports, and emit CLI summary."""
    try:
        cfg = build_config_fn(parse_args_fn())
        scenarios = select_scenarios_fn(load_scenarios_fn(cfg.dataset_path), cfg.scenario_id)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    started_dt = datetime_cls.now(utc_tz)
    started_mono = time.monotonic()

    scenario_results = tuple(run_scenario_fn(cfg, scenario) for scenario in scenarios)
    report = build_report_fn(cfg, scenario_results, started_mono, started_dt)
    write_outputs_fn(report, cfg.output_json, cfg.output_markdown)

    print("Complex scenario suite completed.")
    print(f"  overall={'PASS' if report['overall_passed'] else 'FAIL'}")
    print(
        "  scenarios={passed}/{total}".format(
            passed=report["summary"]["passed"],
            total=report["summary"]["total"],
        )
    )
    print(f"  json_report={cfg.output_json}")
    print(f"  markdown_report={cfg.output_markdown}")

    if not report["overall_passed"]:
        failed = [
            scenario["scenario_id"] for scenario in report["scenarios"] if not scenario["passed"]
        ]
        print(f"  failed_scenarios={failed}")
        return 1

    return 0
