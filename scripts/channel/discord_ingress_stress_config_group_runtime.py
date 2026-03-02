#!/usr/bin/env python3
"""Runtime/concurrency argument groups for Discord ingress stress parser."""

from __future__ import annotations

from typing import Any

from discord_ingress_stress_config_urls import default_ingress_url


def add_runtime_args(parser: Any) -> None:
    """Add runtime and concurrency arguments."""
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
