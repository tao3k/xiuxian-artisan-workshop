#!/usr/bin/env python3
"""Gate and output parser sections for complex scenarios config."""

from __future__ import annotations

from typing import Any


def add_quality_gate_args(parser: Any) -> None:
    """Register complexity/quality gate args."""
    parser.add_argument(
        "--forbid-log-regex",
        action="append",
        default=[],
        help="Regex pattern that must not appear in probe logs (repeatable).",
    )
    parser.add_argument("--min-steps", type=int, default=14, help="Global minimum workflow steps.")
    parser.add_argument(
        "--min-dependency-edges", type=int, default=14, help="Global minimum dependency edges."
    )
    parser.add_argument(
        "--min-critical-path", type=int, default=6, help="Global minimum critical path length."
    )
    parser.add_argument(
        "--min-parallel-waves",
        type=int,
        default=3,
        help="Global minimum count of parallel execution waves.",
    )
    parser.add_argument(
        "--min-error-signals",
        type=int,
        default=2,
        help="Global minimum error-signal steps (tag=error_signal).",
    )
    parser.add_argument(
        "--min-negative-feedback-events",
        type=int,
        default=1,
        help="Global minimum negative feedback events (feedback delta < 0).",
    )
    parser.add_argument(
        "--min-correction-checks",
        type=int,
        default=2,
        help="Global minimum correction-check steps (tag=correction_check).",
    )
    parser.add_argument(
        "--min-successful-corrections",
        type=int,
        default=1,
        help="Global minimum successful correction-check steps after error signals.",
    )
    parser.add_argument(
        "--min-planned-hits",
        type=int,
        default=2,
        help="Global minimum steps where memory recall planning is observed.",
    )
    parser.add_argument(
        "--min-natural-language-steps",
        type=int,
        default=6,
        help="Global minimum count of non-command (natural language) steps.",
    )
    parser.add_argument(
        "--min-recall-credit-events",
        type=int,
        default=0,
        help="Global minimum observed memory recall credit events.",
    )
    parser.add_argument(
        "--min-decay-events",
        type=int,
        default=0,
        help="Global minimum observed memory decay events.",
    )


def add_output_args(parser: Any) -> None:
    """Register report output path args."""
    parser.add_argument(
        "--output-json",
        default=".run/reports/agent-channel-complex-scenarios.json",
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=".run/reports/agent-channel-complex-scenarios.md",
        help="Output Markdown report path.",
    )
