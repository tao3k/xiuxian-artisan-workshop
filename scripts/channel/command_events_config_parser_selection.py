#!/usr/bin/env python3
"""Case/suite/output argument groups for command-events parser."""

from __future__ import annotations

import os
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import argparse


def add_selection_args(parser: argparse.ArgumentParser, *, suites: tuple[str, ...]) -> None:
    """Add suite/case-selection arguments."""
    parser.add_argument(
        "--suite",
        action="append",
        choices=suites,
        default=[],
        help=("Run only selected suite(s): core, control, admin, all. Repeatable. Default: all."),
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        help=(
            "Run only specific case id(s). Repeatable. Use --list-cases to inspect available ids."
        ),
    )
    parser.add_argument(
        "--list-cases",
        action="store_true",
        help="List available black-box case ids and exit.",
    )


def add_output_args(parser: argparse.ArgumentParser) -> None:
    """Add output path arguments."""
    parser.add_argument(
        "--output-json",
        default=os.environ.get(
            "OMNI_COMMAND_EVENTS_OUTPUT_JSON",
            ".run/reports/agent-channel-command-events.json",
        ),
        help="Output JSON report path.",
    )
    parser.add_argument(
        "--output-markdown",
        default=os.environ.get(
            "OMNI_COMMAND_EVENTS_OUTPUT_MARKDOWN",
            ".run/reports/agent-channel-command-events.md",
        ),
        help="Output Markdown report path.",
    )
