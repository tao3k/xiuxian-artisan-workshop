#!/usr/bin/env python3
"""Core primitives for event-sequence checks."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class Reporter:
    """Mutable counters + emit helpers for validation reporting."""

    passes: int = 0
    warnings: int = 0
    failures: int = 0

    def emit_pass(self, message: str) -> None:
        self.passes += 1
        print(f"[PASS] {message}")

    def emit_warn(self, message: str) -> None:
        self.warnings += 1
        print(f"[WARN] {message}")

    def emit_fail(self, message: str) -> None:
        self.failures += 1
        print(f"[FAIL] {message}")


def count_event(lines: list[str], event: str) -> int:
    """Count lines containing a specific event token."""
    return sum(1 for line in lines if event in line)


def first_line(lines: list[str], event: str) -> int:
    """Return first 1-based line index matching event token."""
    for idx, line in enumerate(lines, start=1):
        if event in line:
            return idx
    return 0


def first_line_any(lines: list[str], events: list[str]) -> int:
    """Return first 1-based line index matching any event token."""
    values = [first_line(lines, event) for event in events]
    positives = [value for value in values if value > 0]
    return min(positives) if positives else 0


def check_order(
    reporter: Reporter,
    earlier_label: str,
    earlier_line: int,
    later_label: str,
    later_line: int,
    description: str,
) -> None:
    """Validate event ordering and emit pass/warn/fail."""
    if earlier_line == 0 or later_line == 0:
        reporter.emit_warn(f"{description} (skipped: missing '{earlier_label}' or '{later_label}')")
        return

    if earlier_line < later_line:
        reporter.emit_pass(
            f"{description} (lines: {earlier_label}={earlier_line}, {later_label}={later_line})"
        )
    else:
        reporter.emit_fail(
            f"{description} (unexpected order: {earlier_label}={earlier_line}, "
            f"{later_label}={later_line})"
        )
