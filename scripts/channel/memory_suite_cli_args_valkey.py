#!/usr/bin/env python3
"""Valkey-related CLI args for memory suite."""

from __future__ import annotations

from typing import Any


def add_valkey_args(parser: Any, *, default_valkey_url: str) -> None:
    """Register optional Valkey regression arguments."""
    parser.add_argument(
        "--with-valkey",
        action="store_true",
        help="Run optional Valkey cross-instance memory snapshot continuity check.",
    )
    parser.add_argument(
        "--valkey-url",
        default=default_valkey_url,
        help=f"Valkey URL for optional --with-valkey checks (default: {default_valkey_url}).",
    )
    parser.add_argument(
        "--valkey-prefix",
        default="",
        help=(
            "Optional explicit Valkey key prefix for optional --with-valkey isolation. "
            "Default: an auto-generated per-run prefix."
        ),
    )
