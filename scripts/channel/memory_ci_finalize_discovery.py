#!/usr/bin/env python3
"""Failure report discovery helpers for memory CI finalization."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path


def newest_failure(
    reports_dir: Path,
    profile: str,
    *,
    extension: str,
    start_stamp: int,
) -> tuple[Path | None, int]:
    """Find newest failure report for profile/extension after start stamp."""
    pattern = re.compile(rf"omni-agent-memory-ci-failure-{re.escape(profile)}-(\d+)\.{extension}$")
    best_path: Path | None = None
    best_stamp = -1
    for path in reports_dir.glob(f"omni-agent-memory-ci-failure-{profile}-*.{extension}"):
        match = pattern.match(path.name)
        if match is None:
            continue
        stamp = int(match.group(1))
        if stamp < start_stamp:
            continue
        if stamp > best_stamp:
            best_stamp = stamp
            best_path = path
    return best_path, best_stamp
