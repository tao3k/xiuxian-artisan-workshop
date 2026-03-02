#!/usr/bin/env python3
"""CLI argument parsing for the memory suite script."""

from __future__ import annotations

import argparse

from memory_suite_cli_args_evolution import add_evolution_args
from memory_suite_cli_args_runtime import add_runtime_args
from memory_suite_cli_args_valkey import add_valkey_args


def parse_args(
    *,
    default_max_wait: int,
    default_max_idle_secs: int,
    default_valkey_url: str,
    default_evolution_dataset: str,
    default_evolution_scenario_id: str,
    default_evolution_output_json: str,
    default_evolution_output_markdown: str,
) -> argparse.Namespace:
    """Parse command-line arguments for memory suite orchestration."""
    parser = argparse.ArgumentParser(
        description="Run memory-focused omni-agent Telegram black-box + regression suite."
    )
    add_runtime_args(
        parser,
        default_max_wait=default_max_wait,
        default_max_idle_secs=default_max_idle_secs,
    )
    add_evolution_args(
        parser,
        default_evolution_dataset=default_evolution_dataset,
        default_evolution_scenario_id=default_evolution_scenario_id,
        default_evolution_output_json=default_evolution_output_json,
        default_evolution_output_markdown=default_evolution_output_markdown,
    )
    add_valkey_args(parser, default_valkey_url=default_valkey_url)
    return parser.parse_args()
