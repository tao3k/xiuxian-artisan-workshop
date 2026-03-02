#!/usr/bin/env python3
"""Argument parser builder for runtime trace reconstruction script."""

from __future__ import annotations

import argparse
from pathlib import Path


def parse_args(
    *,
    stage_choices: tuple[str, ...],
    default_required_stages: tuple[str, ...],
) -> argparse.Namespace:
    """Parse CLI arguments for trace reconstruction."""
    parser = argparse.ArgumentParser(
        description="Reconstruct omni-agent runtime trace from log file"
    )
    parser.add_argument("log_file", type=Path, help="Runtime log file path")
    parser.add_argument("--session-id", default="", help="Optional session_id/session_key filter")
    parser.add_argument("--chat-id", type=int, default=None, help="Optional chat_id filter")
    parser.add_argument("--max-events", type=int, default=500, help="Maximum events to include")
    parser.add_argument(
        "--required-stage",
        action="append",
        choices=stage_choices,
        default=[],
        help=(
            "Required lifecycle stage for health evaluation (repeatable). "
            f"Default: {','.join(default_required_stages)}"
        ),
    )
    parser.add_argument(
        "--require-suggested-link",
        action="store_true",
        help="Fail if no suggested_link record appears in filtered trace",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail when chain warnings/errors exist",
    )
    parser.add_argument("--json-out", type=Path, default=None, help="Optional JSON output path")
    parser.add_argument(
        "--markdown-out",
        type=Path,
        default=None,
        help="Optional markdown output path",
    )
    return parser.parse_args()
