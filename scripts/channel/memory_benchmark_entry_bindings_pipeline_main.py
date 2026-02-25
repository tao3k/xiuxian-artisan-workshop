#!/usr/bin/env python3
"""Main-pipeline passthrough binding for memory benchmark entrypoint."""

from __future__ import annotations

from typing import Any


def run_main(
    *,
    runner_module: Any,
    parse_args_value: Any,
    build_config_fn: Any,
    load_scenarios_fn: Any,
    run_mode_fn: Any,
    summarize_mode_fn: Any,
    compare_mode_summaries_fn: Any,
    build_markdown_report_fn: Any,
    build_json_payload_fn: Any,
    write_outputs_fn: Any,
    print_summary_fn: Any,
    to_iso_utc_fn: Any,
) -> int:
    """Run top-level benchmark pipeline."""
    return runner_module.run_benchmark_main(
        parse_args_value,
        build_config_fn=build_config_fn,
        load_scenarios_fn=load_scenarios_fn,
        run_mode_fn=run_mode_fn,
        summarize_mode_fn=summarize_mode_fn,
        compare_mode_summaries_fn=compare_mode_summaries_fn,
        build_markdown_report_fn=build_markdown_report_fn,
        build_json_payload_fn=build_json_payload_fn,
        write_outputs_fn=write_outputs_fn,
        print_summary_fn=print_summary_fn,
        to_iso_utc_fn=to_iso_utc_fn,
    )
