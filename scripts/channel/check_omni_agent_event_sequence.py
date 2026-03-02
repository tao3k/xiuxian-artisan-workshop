#!/usr/bin/env python3
"""
Validate key observability event sequence for Telegram webhook/session/memory flows.

Compatibility target:
  scripts/channel/check-omni-agent-event-sequence.sh
"""

from __future__ import annotations

import importlib
import re

from check_omni_agent_event_sequence_args import parse_args as _parse_args_impl
from check_omni_agent_event_sequence_runtime import run_main as _run_main_impl

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")

_checks_module = importlib.import_module("event_sequence_checks")
Reporter = _checks_module.Reporter


def parse_args():
    """Parse CLI args for event-sequence checker script."""
    return _parse_args_impl()


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
    return _run_main_impl(
        log_file=args.log_file,
        strict=args.strict,
        require_memory=args.require_memory,
        expect_memory_backend=args.expect_memory_backend,
        strip_ansi_fn=strip_ansi,
        run_checks_fn=run_checks,
    )


if __name__ == "__main__":
    raise SystemExit(main())
