#!/usr/bin/env python3
"""Execution orchestration for memory benchmark script."""

from __future__ import annotations

import subprocess
import sys
import time
from typing import Any


def run_benchmark_main(
    args: Any,
    *,
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
    """Run full benchmark flow and persist JSON/Markdown outputs."""
    try:
        config = build_config_fn(args)
        scenarios = load_scenarios_fn(config.dataset_path)
    except ValueError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2

    started_ts = time.time()
    started_at = to_iso_utc_fn(started_ts)

    print("Running memory benchmark...", flush=True)
    print(f"dataset={config.dataset_path}", flush=True)
    print(f"log_file={config.log_file}", flush=True)
    print(
        f"session_target=chat:{config.chat_id} user:{config.user_id} "
        f"thread:{config.thread_id if config.thread_id is not None else 'none'}",
        flush=True,
    )
    print(
        f"runtime_partition_mode={config.runtime_partition_mode or 'unknown'}",
        flush=True,
    )
    print(f"modes={config.modes}", flush=True)
    print(f"iterations={config.iterations}", flush=True)
    print(f"fail_on_mcp_error={config.fail_on_mcp_error}", flush=True)
    print(
        f"feedback_policy={config.feedback_policy} "
        f"feedback_down_threshold={config.feedback_down_threshold}",
        flush=True,
    )

    mode_turns: dict[str, list[Any]] = {}
    try:
        for mode in config.modes:
            mode_turns[mode] = run_mode_fn(config, scenarios, mode)
    except RuntimeError as error:
        print(f"Error: {error}", file=sys.stderr)
        return 2
    except subprocess.CalledProcessError as error:
        print(f"Error: benchmark probe failed with exit code {error.returncode}", file=sys.stderr)
        return error.returncode if error.returncode != 0 else 1

    mode_summaries = {
        mode: summarize_mode_fn(
            mode=mode,
            iterations=config.iterations,
            scenario_count=len(scenarios),
            turns=turns,
        )
        for mode, turns in mode_turns.items()
    }

    comparison: dict[str, float] | None = None
    if "baseline" in mode_summaries and "adaptive" in mode_summaries:
        comparison = compare_mode_summaries_fn(
            mode_summaries["baseline"],
            mode_summaries["adaptive"],
        )

    finished_ts = time.time()
    finished_at = to_iso_utc_fn(finished_ts)

    markdown = build_markdown_report_fn(
        config=config,
        scenarios=scenarios,
        started_at=started_at,
        finished_at=finished_at,
        mode_summaries=mode_summaries,
        comparison=comparison,
    )

    json_payload = build_json_payload_fn(
        config=config,
        scenarios=scenarios,
        mode_summaries=mode_summaries,
        comparison=comparison,
        mode_turns=mode_turns,
        started_at=started_at,
        finished_at=finished_at,
        duration_secs=finished_ts - started_ts,
    )
    write_outputs_fn(
        config=config,
        json_payload=json_payload,
        markdown=markdown,
    )
    print_summary_fn(
        config=config,
        mode_summaries=mode_summaries,
        comparison=comparison,
    )

    return 0
