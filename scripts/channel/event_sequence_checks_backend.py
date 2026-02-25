#!/usr/bin/env python3
"""Memory-backend expectation checks for event sequence validation."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from event_sequence_checks_core import Reporter


def check_expected_memory_backend(
    reporter: Reporter,
    stripped_lines: list[str],
    *,
    expect_memory_backend: str,
) -> None:
    """Validate configured memory backend expectation against log lines."""
    if not expect_memory_backend:
        return

    expected_backend_re = re.compile(
        rf'backend="?{re.escape(expect_memory_backend)}"?\b|'
        rf'"backend"\s*:\s*"{re.escape(expect_memory_backend)}"'
    )
    count_expected_backend = sum(
        1
        for line in stripped_lines
        if "agent.memory.backend.initialized" in line and expected_backend_re.search(line)
    )
    if count_expected_backend > 0:
        reporter.emit_pass(
            "memory backend expectation matched "
            f"(expected={expect_memory_backend}, count={count_expected_backend})"
        )
    else:
        reporter.emit_fail(
            f"memory backend expectation not matched (expected={expect_memory_backend})"
        )
