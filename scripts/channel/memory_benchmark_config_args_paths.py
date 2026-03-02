#!/usr/bin/env python3
"""Path helpers for memory benchmark CLI config."""

from __future__ import annotations

import os
from pathlib import Path


def default_report_path(filename: str) -> Path:
    """Build a report path under PRJ runtime report directory."""
    runtime_root = Path(os.environ.get("PRJ_RUNTIME_DIR", ".run"))
    if not runtime_root.is_absolute():
        project_root = Path(os.environ.get("PRJ_ROOT", Path.cwd()))
        runtime_root = project_root / runtime_root
    return runtime_root / "reports" / filename
