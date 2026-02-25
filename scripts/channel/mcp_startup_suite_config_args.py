#!/usr/bin/env python3
"""Argument parsing helpers for MCP startup suite runner."""

from __future__ import annotations

import argparse
from pathlib import Path

from path_resolver import default_report_path, project_root_from


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments for startup suite."""
    parser = argparse.ArgumentParser(
        description="Run hot/cold MCP startup regression suite for omni-agent gateway."
    )
    parser.add_argument("--hot-rounds", type=int, default=20)
    parser.add_argument("--hot-parallel", type=int, default=8)
    parser.add_argument("--cold-rounds", type=int, default=8)
    parser.add_argument("--cold-parallel", type=int, default=4)
    parser.add_argument("--startup-timeout-secs", type=int, default=60)
    parser.add_argument("--cooldown-secs", type=float, default=0.2)
    parser.add_argument("--mcp-host", default="127.0.0.1")
    parser.add_argument("--mcp-port", type=int, default=3002)
    parser.add_argument("--mcp-config", default=".mcp.json")
    parser.add_argument("--health-url", default="")
    parser.add_argument("--strict-health-check", action="store_true")
    parser.add_argument("--no-strict-health-check", action="store_true")
    parser.add_argument("--health-probe-interval-secs", type=float, default=0.2)
    parser.add_argument("--health-probe-timeout-secs", type=float, default=1.0)
    parser.add_argument("--restart-mcp-cmd", default="")
    parser.add_argument(
        "--allow-mcp-restart",
        action="store_true",
        help=(
            "Allow this suite to restart the shared MCP process for cold-start checks. "
            "By default cold mode is auto-skipped to avoid disrupting a live MCP instance."
        ),
    )
    parser.add_argument("--restart-mcp-settle-secs", type=float, default=0.2)
    parser.add_argument("--restart-health-timeout-secs", type=int, default=30)
    parser.add_argument("--restart-no-embedding", action="store_true")
    parser.add_argument("--skip-hot", action="store_true")
    parser.add_argument("--skip-cold", action="store_true")
    parser.add_argument("--quality-max-failed-probes", type=int, default=0)
    parser.add_argument("--quality-max-hot-p95-ms", type=float, default=1200.0)
    parser.add_argument("--quality-max-cold-p95-ms", type=float, default=1500.0)
    parser.add_argument("--quality-min-health-samples", type=int, default=1)
    parser.add_argument("--quality-max-health-failure-rate", type=float, default=0.02)
    parser.add_argument("--quality-max-health-p95-ms", type=float, default=350.0)
    parser.add_argument("--quality-baseline-json", default="")
    parser.add_argument("--quality-max-hot-p95-regression-ratio", type=float, default=0.5)
    parser.add_argument("--quality-max-cold-p95-regression-ratio", type=float, default=0.5)
    parser.add_argument(
        "--project-root",
        default=str(project_root_from(Path.cwd())),
        help="Project root (default: auto-detect from .git).",
    )
    parser.add_argument(
        "--output-json",
        default=str(default_report_path("omni-agent-mcp-startup-suite.json")),
    )
    parser.add_argument(
        "--output-markdown",
        default=str(default_report_path("omni-agent-mcp-startup-suite.md")),
    )
    return parser.parse_args()
