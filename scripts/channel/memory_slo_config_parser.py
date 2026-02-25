#!/usr/bin/env python3
"""CLI parser helpers for memory/session SLO report config."""

from __future__ import annotations

import argparse
from pathlib import Path

from memory_slo_config_paths import default_report_path, project_root_from


def parse_required_modes(raw: str) -> tuple[str, ...]:
    """Parse required benchmark modes from comma-separated input."""
    modes = tuple(mode.strip() for mode in raw.split(",") if mode.strip())
    if not modes:
        raise ValueError("--required-benchmark-modes must contain at least one mode.")
    return modes


def parse_args() -> argparse.Namespace:
    """Parse CLI arguments for SLO report generation."""
    parser = argparse.ArgumentParser(
        description=(
            "Build a unified memory/session SLO report from evolution, benchmark, "
            "and session-matrix outputs."
        )
    )
    parser.add_argument(
        "--project-root",
        default=str(project_root_from(Path.cwd())),
        help="Project root (default: auto-detect from .git).",
    )
    parser.add_argument(
        "--evolution-report-json",
        default=str(default_report_path("omni-agent-memory-evolution.json")),
    )
    parser.add_argument(
        "--benchmark-report-json",
        default=str(default_report_path("omni-agent-memory-benchmark.json")),
    )
    parser.add_argument(
        "--session-matrix-report-json",
        default=str(default_report_path("agent-channel-session-matrix.json")),
    )
    parser.add_argument(
        "--runtime-log-file",
        default="",
        help="Optional runtime log for stream ack/read-failure checks.",
    )
    parser.add_argument(
        "--output-json",
        default=str(default_report_path("omni-agent-memory-slo-report.json")),
    )
    parser.add_argument(
        "--output-markdown",
        default=str(default_report_path("omni-agent-memory-slo-report.md")),
    )
    parser.add_argument("--min-planned-hits", type=int, default=10)
    parser.add_argument("--min-successful-corrections", type=int, default=3)
    parser.add_argument("--min-recall-credit-events", type=int, default=1)
    parser.add_argument("--min-quality-score", type=float, default=90.0)
    parser.add_argument("--required-benchmark-modes", default="baseline,adaptive")
    parser.add_argument("--min-query-turns", type=int, default=1)
    parser.add_argument("--max-mode-mcp-error-turns", type=int, default=0)
    parser.add_argument("--max-total-mcp-error-turns", type=int, default=0)
    parser.add_argument("--min-session-steps", type=int, default=1)
    parser.add_argument("--max-session-failed-steps", type=int, default=0)
    parser.add_argument("--enable-stream-gate", action="store_true")
    parser.add_argument("--min-stream-ack-ratio", type=float, default=0.999)
    parser.add_argument("--min-stream-published-events", type=int, default=1)
    parser.add_argument("--max-stream-read-failed", type=int, default=0)
    return parser.parse_args()
