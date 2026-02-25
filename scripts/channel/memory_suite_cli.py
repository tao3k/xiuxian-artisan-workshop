#!/usr/bin/env python3
"""CLI argument parsing for the memory suite script."""

from __future__ import annotations

import argparse
import os


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
    parser.add_argument(
        "--suite",
        choices=("quick", "full"),
        default="quick",
        help="Suite mode: quick (black-box only) or full (black-box + cargo regressions).",
    )
    parser.add_argument(
        "--max-wait",
        type=int,
        default=default_max_wait,
        help=f"Per black-box probe max wait in seconds (default: {default_max_wait}).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=default_max_idle_secs,
        help=f"Per black-box probe max idle seconds (default: {default_max_idle_secs}).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME", ""),
        help="Synthetic Telegram username for allowlist checks.",
    )
    parser.add_argument(
        "--require-live-turn",
        action="store_true",
        help=(
            "Also probe a normal non-command turn and require memory recall observability "
            "events in logs."
        ),
    )
    parser.add_argument(
        "--skip-blackbox",
        action="store_true",
        help="Skip webhook black-box checks (useful when local webhook runtime is not running).",
    )
    parser.add_argument(
        "--skip-rust",
        action="store_true",
        help="Skip Rust regression checks.",
    )
    parser.add_argument(
        "--skip-evolution",
        action="store_true",
        help=(
            "Skip memory self-evolution DAG black-box scenario in full suite. "
            "By default, full suite includes this scenario."
        ),
    )
    parser.add_argument(
        "--evolution-dataset",
        default=default_evolution_dataset,
        help="Path to memory self-evolution complex scenario dataset JSON.",
    )
    parser.add_argument(
        "--evolution-scenario",
        default=default_evolution_scenario_id,
        help=(
            "Scenario id to run from the evolution dataset "
            f"(default: {default_evolution_scenario_id})."
        ),
    )
    parser.add_argument(
        "--evolution-max-parallel",
        type=int,
        default=1,
        help="Max parallel probes per wave for evolution scenario (default: 1).",
    )
    parser.add_argument(
        "--evolution-output-json",
        default=default_evolution_output_json,
        help="Output JSON report path for evolution scenario.",
    )
    parser.add_argument(
        "--evolution-output-markdown",
        default=default_evolution_output_markdown,
        help="Output Markdown report path for evolution scenario.",
    )
    parser.add_argument(
        "--with-valkey",
        action="store_true",
        help="Run optional Valkey cross-instance memory snapshot continuity check.",
    )
    parser.add_argument(
        "--valkey-url",
        default=default_valkey_url,
        help=f"Valkey URL for optional --with-valkey checks (default: {default_valkey_url}).",
    )
    parser.add_argument(
        "--valkey-prefix",
        default="",
        help=(
            "Optional explicit Valkey key prefix for optional --with-valkey isolation. "
            "Default: an auto-generated per-run prefix."
        ),
    )
    return parser.parse_args()
