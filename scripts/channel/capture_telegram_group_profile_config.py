#!/usr/bin/env python3
"""CLI parsing and small helpers for Telegram group profile capture."""

from __future__ import annotations

import argparse


def parse_args() -> argparse.Namespace:
    """Parse CLI args for group profile capture utility."""
    parser = argparse.ArgumentParser(
        description=(
            "Persist Telegram group mappings for black-box tests. "
            "Reads webhook logs and writes JSON/env profile files."
        )
    )
    parser.add_argument(
        "--titles",
        default="Test1,Test2,Test3",
        help="Comma-separated group titles in desired A/B/C order (default: Test1,Test2,Test3).",
    )
    parser.add_argument(
        "--log-file",
        default=".run/logs/omni-agent-webhook.log",
        help="Webhook runtime log file path.",
    )
    parser.add_argument(
        "--output-json",
        default=".run/config/agent-channel-groups.json",
        help="Output profile JSON path.",
    )
    parser.add_argument(
        "--output-env",
        default=".run/config/agent-channel-groups.env",
        help="Output profile env file path.",
    )
    parser.add_argument(
        "--user-id",
        type=int,
        default=None,
        help=(
            "Optional fixed Telegram user id for all sessions "
            "(default: inferred from matched session_key values)."
        ),
    )
    parser.add_argument(
        "--allow-missing",
        action="store_true",
        help="Allow missing titles and write only discovered mappings.",
    )
    return parser.parse_args()


def normalize_title(value: str) -> str:
    """Normalize titles for case-insensitive matching."""
    return value.strip().casefold()


def parse_user_id(session_key: str) -> int | None:
    """Parse user ID from chat:user or chat:thread:user session keys."""
    parts = session_key.split(":")
    if len(parts) == 2:
        return int(parts[1])
    if len(parts) == 3:
        return int(parts[2])
    return None
