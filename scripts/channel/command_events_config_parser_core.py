#!/usr/bin/env python3
"""Core CLI argument group for command-events parser."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse


def add_core_args(parser: argparse.ArgumentParser) -> None:
    """Add core command-events runner arguments."""
    parser.add_argument(
        "--max-wait",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_WAIT_SECS", "25")),
        help="Overall wait upper-bound per probe in seconds (default: 25).",
    )
    parser.add_argument(
        "--max-idle-secs",
        type=int,
        default=int(os.environ.get("OMNI_BLACKBOX_MAX_IDLE_SECS", "25")),
        help="Max idle wait for new logs per probe in seconds (default: 25).",
    )
    parser.add_argument(
        "--username",
        default=os.environ.get("OMNI_TEST_USERNAME", ""),
        help="Synthetic Telegram username (default: $OMNI_TEST_USERNAME).",
    )
    parser.add_argument(
        "--allow-chat-id",
        action="append",
        default=[],
        help=("Allowlisted chat id passed through to agent_channel_blackbox.py (repeatable)."),
    )
