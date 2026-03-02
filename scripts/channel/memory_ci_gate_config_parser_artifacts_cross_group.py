#!/usr/bin/env python3
"""Cross-group scenario argument registration for memory CI gate parser."""

from __future__ import annotations

from typing import Any


def add_cross_group_args(parser: Any, *, project_root: Any) -> None:
    """Register cross-group scenario dataset and runtime threshold arguments."""
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
