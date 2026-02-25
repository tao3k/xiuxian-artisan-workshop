#!/usr/bin/env python3
"""Top-level main binding for complex scenarios runner."""

from __future__ import annotations

from typing import Any


def run_main(
    *,
    runner_module: Any,
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
    """Run the top-level complex scenarios pipeline."""
    return runner_module.run_complex_scenarios_main(
        parse_args_fn=parse_args_fn,
        build_config_fn=build_config_fn,
        load_scenarios_fn=load_scenarios_fn,
        select_scenarios_fn=select_scenarios_fn,
        run_scenario_fn=run_scenario_fn,
        build_report_fn=build_report_fn,
        write_outputs_fn=write_outputs_fn,
        datetime_cls=datetime_cls,
        utc_tz=utc_tz,
    )
