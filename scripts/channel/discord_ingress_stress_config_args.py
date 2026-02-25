#!/usr/bin/env python3
"""CLI argument parsing for Discord ingress stress probe config."""

from __future__ import annotations

import argparse
import os
from pathlib import Path

from discord_ingress_stress_config_urls import default_ingress_url
from path_resolver import default_report_path, project_root_from


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments for Discord ingress stress probe."""
    parser = argparse.ArgumentParser(
        description=(
            "Stress Discord ingress by posting concurrent synthetic events and "
            "measuring HTTP/latency and queue-pressure signals."
        )
    )
    parser.add_argument("--rounds", type=int, default=6, help="Measured rounds (default: 6).")
    parser.add_argument(
        "--warmup-rounds",
        type=int,
        default=1,
        help="Warmup rounds excluded from quality gate (default: 1).",
    )
    parser.add_argument(
        "--parallel",
        type=int,
        default=8,
        help="Concurrent workers per round (default: 8).",
    )
    parser.add_argument(
        "--requests-per-worker",
        type=int,
        default=20,
        help="Requests sent by each worker each round (default: 20).",
    )
    parser.add_argument(
        "--timeout-secs",
        type=float,
        default=10.0,
        help="HTTP request timeout in seconds (default: 10).",
    )
    parser.add_argument(
        "--cooldown-secs",
        type=float,
        default=0.2,
        help="Sleep interval between rounds in seconds (default: 0.2).",
    )
    parser.add_argument(
        "--ingress-url",
        default=default_ingress_url(),
        help="Discord ingress URL (default: env/bind fallback).",
    )
    parser.add_argument(
        "--secret-token",
        default=os.environ.get("OMNI_TEST_DISCORD_INGRESS_SECRET", "").strip(),
        help="Optional ingress secret token for header x-omni-discord-ingress-token.",
    )
    parser.add_argument(
        "--channel-id",
        default=os.environ.get("OMNI_TEST_DISCORD_CHANNEL_ID", "").strip(),
        help="Synthetic Discord channel_id.",
    )
    parser.add_argument(
        "--user-id",
        default=os.environ.get("OMNI_TEST_DISCORD_USER_ID", "").strip(),
        help="Synthetic Discord user_id.",
    )
    parser.add_argument(
        "--guild-id",
        default=os.environ.get("OMNI_TEST_DISCORD_GUILD_ID", "").strip(),
        help="Optional synthetic guild_id.",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_DISCORD_USERNAME", "").strip(),
        help="Optional synthetic Discord username.",
    )
    parser.add_argument(
        "--role-id",
        action="append",
        default=[],
        help="Optional synthetic member role id (repeatable).",
    )
    parser.add_argument(
        "--prompt",
        default="stress ingress probe",
        help="Synthetic message content sent in ingress event payload.",
    )
    parser.add_argument(
        "--log-file",
        default=os.environ.get("OMNI_CHANNEL_LOG_FILE", ".run/logs/omni-agent-webhook.log"),
        help="Runtime log file used to count queue pressure events.",
    )
    parser.add_argument(
        "--project-root",
        default=str(project_root_from(Path.cwd())),
        help="Project root for resolving relative paths.",
    )
    parser.add_argument(
        "--output-json",
        default=str(default_report_path("omni-agent-discord-ingress-stress.json")),
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=str(default_report_path("omni-agent-discord-ingress-stress.md")),
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
    return parser.parse_args()
