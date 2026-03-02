#!/usr/bin/env python3
"""Report/artifact path argument registration for memory CI gate parser."""

from __future__ import annotations

from typing import Any


def add_report_path_args(parser: Any) -> None:
    """Register runtime/evolution/benchmark/session-matrix/trace report arguments."""
    parser.add_argument(
        "--runtime-log-file",
        default="",
        help=(
            "Optional runtime log file path. "
            "Default: .run/logs/omni-agent-webhook-ci-<profile>-<run-suffix>.log"
        ),
    )
    parser.add_argument(
        "--mock-log-file",
        default="",
        help=(
            "Optional mock Telegram log file path. "
            "Default: .run/logs/omni-agent-mock-telegram-<profile>-<run-suffix>.log"
        ),
    )
    parser.add_argument(
        "--evolution-report-json",
        default="",
        help=(
            "Optional evolution report path. "
            "Default: .run/reports/omni-agent-memory-evolution-<profile>-<run-suffix>.json"
        ),
    )
    parser.add_argument(
        "--benchmark-report-json",
        default="",
        help=(
            "Optional benchmark report path. "
            "Default: .run/reports/omni-agent-memory-benchmark-<profile>-<run-suffix>.json"
        ),
    )
    parser.add_argument(
        "--session-matrix-report-json",
        default="",
        help=(
            "Optional session matrix JSON path. "
            "Default: .run/reports/agent-channel-session-matrix-<profile>-<run-suffix>.json"
        ),
    )
    parser.add_argument(
        "--session-matrix-report-markdown",
        default="",
        help=(
            "Optional session matrix Markdown path. "
            "Default: .run/reports/agent-channel-session-matrix-<profile>-<run-suffix>.md"
        ),
    )
    parser.add_argument(
        "--trace-report-json",
        default="",
        help=(
            "Optional trace report JSON path. "
            "Default: .run/reports/omni-agent-trace-reconstruction-<profile>-<run-suffix>.json"
        ),
    )
    parser.add_argument(
        "--trace-report-markdown",
        default="",
        help=(
            "Optional trace report Markdown path. "
            "Default: .run/reports/omni-agent-trace-reconstruction-<profile>-<run-suffix>.md"
        ),
    )
    parser.add_argument(
        "--cross-group-report-json",
        default="",
        help=(
            "Optional cross-group mixed-concurrency report JSON path. "
            "Default: .run/reports/agent-channel-cross-group-complex-<profile>-<run-suffix>.json"
        ),
    )
    parser.add_argument(
        "--cross-group-report-markdown",
        default="",
        help=(
            "Optional cross-group mixed-concurrency report Markdown path. "
            "Default: .run/reports/agent-channel-cross-group-complex-<profile>-<run-suffix>.md"
        ),
    )
