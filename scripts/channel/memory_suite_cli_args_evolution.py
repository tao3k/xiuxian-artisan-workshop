#!/usr/bin/env python3
"""Evolution-scenario CLI args for memory suite."""

from __future__ import annotations

from typing import Any


def add_evolution_args(
    parser: Any,
    *,
    default_evolution_dataset: str,
    default_evolution_scenario_id: str,
    default_evolution_output_json: str,
    default_evolution_output_markdown: str,
) -> None:
    """Register evolution scenario arguments."""
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
