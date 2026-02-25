#!/usr/bin/env python3
"""CLI parser for runtime monitor."""

from __future__ import annotations

import argparse


def parse_args() -> argparse.Namespace:
    """Parse runtime monitor CLI args."""
    parser = argparse.ArgumentParser(
        description="Run a command with structured exit/observability reporting."
    )
    parser.add_argument("--log-file", required=True, help="Plain-text streaming log output file.")
    parser.add_argument(
        "--report-file",
        required=True,
        help="Structured JSON exit report file (overwritten per run).",
    )
    parser.add_argument(
        "--report-jsonl",
        default="",
        help="Optional JSONL file to append one report entry per run.",
    )
    parser.add_argument(
        "--tail-lines",
        type=int,
        default=40,
        help="Number of trailing process output lines to include in report.",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="Command to execute (prefix with --).",
    )
    args = parser.parse_args()
    if args.command and args.command[0] == "--":
        args.command = args.command[1:]
    if not args.command:
        parser.error("No command provided. Use `-- <command> [args...]`.")
    return args
