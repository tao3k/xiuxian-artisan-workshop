#!/usr/bin/env python3
"""Statistics/tail helpers for dedup probe runtime flow."""

from __future__ import annotations

import sys
from typing import Any


def collect_stats(lines: list[str], update_id: int, *, strip_ansi_fn: Any) -> dict[str, int]:
    """Collect accepted/duplicate/evaluated stats for one update id."""
    normalized = [strip_ansi_fn(line) for line in lines]
    accepted = [
        idx
        for idx, line in enumerate(normalized, start=1)
        if 'event="telegram.dedup.update_accepted"' in line and f"update_id={update_id}" in line
    ]
    duplicate = [
        idx
        for idx, line in enumerate(normalized, start=1)
        if 'event="telegram.dedup.duplicate_detected"' in line and f"update_id={update_id}" in line
    ]
    evaluated = [
        line
        for line in normalized
        if 'event="telegram.dedup.evaluated"' in line and f"update_id={update_id}" in line
    ]
    evaluated_true = sum("duplicate=true" in line for line in evaluated)
    evaluated_false = sum("duplicate=false" in line for line in evaluated)
    return {
        "accepted_count": len(accepted),
        "duplicate_count": len(duplicate),
        "accepted_line": accepted[0] if accepted else 0,
        "duplicate_line": duplicate[0] if duplicate else 0,
        "evaluated_total": len(evaluated),
        "evaluated_true": evaluated_true,
        "evaluated_false": evaluated_false,
    }


def print_relevant_tail(lines: list[str], update_id: int, *, strip_ansi_fn: Any) -> None:
    """Print tail lines relevant to dedup events/update id."""
    relevant = [
        strip_ansi_fn(line)
        for line in lines
        if "telegram.dedup." in line or f"update_id={update_id}" in line
    ]
    print("Relevant tail:", file=sys.stderr)
    for line in relevant[-60:]:
        print(f"  {line}", file=sys.stderr)
