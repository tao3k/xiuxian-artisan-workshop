#!/usr/bin/env python3
"""Runtime entry helpers for event-sequence checker script."""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

from log_io import iter_log_lines


def run_main(
    *,
    log_file: str,
    strict: bool,
    require_memory: bool,
    expect_memory_backend: str,
    strip_ansi_fn: Any,
    run_checks_fn: Any,
) -> int:
    """Load log lines and execute event-sequence checks."""
    path = Path(log_file)
    if not path.is_file():
        print(f"Error: log file not found: {path}", file=sys.stderr)
        return 2

    lines: list[str] = []
    stripped_lines: list[str] = []
    for line in iter_log_lines(path, errors="replace"):
        lines.append(line)
        stripped_lines.append(strip_ansi_fn(line))
    return run_checks_fn(
        lines=lines,
        stripped_lines=stripped_lines,
        strict=strict,
        require_memory=require_memory,
        expect_memory_backend=expect_memory_backend,
    )
