#!/usr/bin/env python3
"""Output/quality argument groups for Discord ingress stress parser."""

from __future__ import annotations

from typing import Any

import discord_ingress_stress_config_args_env as _env


def add_output_and_quality_args(parser: Any) -> None:
    """Add log/report paths and quality gate arguments."""
    parser.add_argument(
        "--log-file",
        default=_env.default_log_file(),
        help="Runtime log file used to count queue pressure events.",
    )
    parser.add_argument(
        "--project-root",
        default=_env.default_project_root(),
        help="Project root for resolving relative paths.",
    )
    parser.add_argument(
        "--output-json",
        default=_env.default_output_json(),
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=_env.default_output_markdown(),
        help="Output Markdown report path.",
    )
    parser.add_argument(
        "--quality-max-failure-rate",
        type=float,
        default=0.0,
        help="Maximum allowed failed_request/total_request ratio (default: 0).",
    )
    parser.add_argument(
        "--quality-max-p95-ms",
        type=float,
        default=0.0,
        help="Maximum allowed measured-round p95 latency in ms (<=0 disables gate).",
    )
    parser.add_argument(
        "--quality-min-rps",
        type=float,
        default=0.0,
        help="Minimum allowed measured-round average RPS (<=0 disables gate).",
    )
