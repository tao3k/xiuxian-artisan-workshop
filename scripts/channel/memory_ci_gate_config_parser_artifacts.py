#!/usr/bin/env python3
"""Artifact/report argument sections for memory CI gate parser."""

from __future__ import annotations

from typing import Any


def add_artifact_args(parser: Any, *, project_root: Any) -> None:
    """Register artifact and report path arguments."""
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
    parser.add_argument(
        "--cross-group-dataset",
        default=str(
            project_root / "scripts" / "channel" / "fixtures" / "complex_blackbox_scenarios.json"
        ),
        help="Dataset path for the cross-group mixed-concurrency complex scenario.",
    )
    parser.add_argument(
        "--cross-group-scenario",
        default="cross_group_control_plane_stress",
        help="Scenario id for cross-group mixed-concurrency gate.",
    )
    parser.add_argument(
        "--cross-group-max-wait",
        type=int,
        default=90,
        help="Cross-group complex scenario per-step max wait (seconds).",
    )
    parser.add_argument(
        "--cross-group-max-idle",
        type=int,
        default=80,
        help="Cross-group complex scenario max idle wait (seconds).",
    )
    parser.add_argument(
        "--cross-group-max-parallel",
        type=int,
        default=3,
        help="Cross-group complex scenario max parallel probes per wave.",
    )
