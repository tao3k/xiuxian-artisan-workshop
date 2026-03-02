#!/usr/bin/env python3
"""Runtime/output/policy argument groups for memory benchmark parser."""

from __future__ import annotations

from typing import Any

from memory_benchmark_config_args_paths import default_report_path


def add_runtime_args(
    parser: Any,
    *,
    default_max_wait: int,
    default_max_idle_secs: int,
) -> None:
    """Add runtime loop and mode arguments."""
    parser.add_argument(
        "--max-wait",
        type=int,
        default=default_max_wait,
        help=f"Per probe max wait in seconds (default: {default_max_wait}).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=default_max_idle_secs,
        help=f"Per probe max idle seconds (default: {default_max_idle_secs}).",
    )
    parser.add_argument(
        "--mode",
        action="append",
        choices=("baseline", "adaptive"),
        help="Benchmark mode (repeatable). Default: baseline+adaptive.",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=1,
        help="Dataset replay count per mode (default: 1).",
    )
    parser.add_argument(
        "--skip-reset",
        action="store_true",
        help="Do not issue /reset between scenarios.",
    )


def add_output_and_policy_args(parser: Any) -> None:
    """Add output report and policy controls."""
    parser.add_argument(
        "--output-json",
        default=str(default_report_path("omni-agent-memory-benchmark.json")),
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=str(default_report_path("omni-agent-memory-benchmark.md")),
        help="Output Markdown report path.",
    )
    parser.add_argument(
        "--fail-on-mcp-error",
        action="store_true",
        help=(
            "Fail immediately when logs include `tools/call: Mcp error`. "
            "Default behavior records this as interference."
        ),
    )
    parser.add_argument(
        "--feedback-policy",
        choices=("strict", "deadband"),
        default="deadband",
        help=(
            "Adaptive feedback policy. "
            "`strict`: success=up, failure=down. "
            "`deadband`: success=up, strong failure only=down, else neutral."
        ),
    )
    parser.add_argument(
        "--feedback-down-threshold",
        type=float,
        default=0.34,
        help=(
            "When feedback-policy=deadband, send `down` only when keyword hit ratio "
            "is less than or equal to this threshold (default: 0.34)."
        ),
    )
