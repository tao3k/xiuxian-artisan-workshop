#!/usr/bin/env python3
"""
Validate key observability event sequence for Telegram webhook/session/memory flows.

Compatibility target:
  scripts/channel/check-omni-agent-event-sequence.sh
"""

from __future__ import annotations

import argparse
import importlib
import re
import sys
from pathlib import Path

from log_io import iter_log_lines

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")

_checks_module = importlib.import_module("event_sequence_checks")
Reporter = _checks_module.Reporter


def parse_args() -> argparse.Namespace:
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


def strip_ansi(text: str) -> str:
    return ANSI_RE.sub("", text)


def count_event(lines: list[str], event: str) -> int:
    return _checks_module.count_event(lines, event)


def first_line(lines: list[str], event: str) -> int:
    return _checks_module.first_line(lines, event)


def first_line_any(lines: list[str], events: list[str]) -> int:
    return _checks_module.first_line_any(lines, events)


def check_order(
    reporter: Reporter,
    earlier_label: str,
    earlier_line: int,
    later_label: str,
    later_line: int,
    description: str,
) -> None:
    _checks_module.check_order(
        reporter,
        earlier_label,
        earlier_line,
        later_label,
        later_line,
        description,
    )


def run_checks(
    lines: list[str],
    stripped_lines: list[str],
    strict: bool,
    require_memory: bool,
    expect_memory_backend: str,
) -> int:
    return _checks_module.run_checks(
        lines=lines,
        stripped_lines=stripped_lines,
        strict=strict,
        require_memory=require_memory,
        expect_memory_backend=expect_memory_backend,
    )


def main() -> int:
    args = parse_args()
    log_file = Path(args.log_file)
    if not log_file.is_file():
        print(f"Error: log file not found: {log_file}", file=sys.stderr)
        return 2

    lines: list[str] = []
    stripped_lines: list[str] = []
    for line in iter_log_lines(log_file, errors="replace"):
        lines.append(line)
        stripped_lines.append(strip_ansi(line))
    return run_checks(
        lines=lines,
        stripped_lines=stripped_lines,
        strict=args.strict,
        require_memory=args.require_memory,
        expect_memory_backend=args.expect_memory_backend,
    )


if __name__ == "__main__":
    raise SystemExit(main())
