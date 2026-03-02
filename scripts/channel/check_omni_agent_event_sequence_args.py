#!/usr/bin/env python3
"""CLI argument parsing for event-sequence checks."""

from __future__ import annotations

import argparse


def parse_args() -> argparse.Namespace:
    """Parse CLI args for event-sequence checker script."""
    parser = argparse.ArgumentParser(
        prog="check_omni_agent_event_sequence.py",
        description=(
            "Validate key observability event sequence for Telegram webhook + session gate + "
            "memory persistence flows."
        ),
    )
    parser.add_argument("log_file", help="Path to agent log file.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat warnings as failures.",
    )
    parser.add_argument(
        "--require-memory",
        action="store_true",
        help="Fail if memory backend lifecycle events are missing.",
    )
    parser.add_argument(
        "--expect-memory-backend",
        choices=("local", "valkey"),
        default="",
        help="Require backend in agent.memory.backend.initialized to match.",
    )
    return parser.parse_args()
