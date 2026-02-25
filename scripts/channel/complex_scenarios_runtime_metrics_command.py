#!/usr/bin/env python3
"""Command execution metric helpers for complex scenario runtime probes."""

from __future__ import annotations

import subprocess
import time


def run_cmd(cmd: list[str]) -> tuple[int, int, str, str]:
    """Execute one blackbox probe command."""
    started = time.monotonic()
    completed = subprocess.run(cmd, capture_output=True, text=True, check=False)
    duration_ms = int((time.monotonic() - started) * 1000)
    return completed.returncode, duration_ms, completed.stdout, completed.stderr
