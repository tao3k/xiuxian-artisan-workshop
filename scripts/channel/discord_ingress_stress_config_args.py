#!/usr/bin/env python3
"""CLI argument parsing for Discord ingress stress probe config."""

from __future__ import annotations

import argparse

from discord_ingress_stress_config_args_groups import (
    add_identity_args,
    add_output_and_quality_args,
    add_runtime_args,
)


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments for Discord ingress stress probe."""
    parser = argparse.ArgumentParser(
        description=(
            "Stress Discord ingress by posting concurrent synthetic events and "
            "measuring HTTP/latency and queue-pressure signals."
        )
    )
    add_runtime_args(parser)
    add_identity_args(parser)
    add_output_and_quality_args(parser)
    return parser.parse_args()
