#!/usr/bin/env python3
"""Runtime helpers for acceptance runner step execution."""

from __future__ import annotations

import subprocess
import time
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def tail_text(value: str, max_lines: int = 40) -> str:
    """Return tail text limited by line count."""
    lines = value.splitlines()
    if len(lines) <= max_lines:
        return value
    return "\n".join(lines[-max_lines:])


def run_step(
    *,
    step: str,
    title: str,
    cmd: list[str],
    expected_outputs: list[Path],
    attempts: int,
    step_result_cls: Any,
) -> Any:
    """Run one pipeline step command and build typed result."""
    started = time.perf_counter()
    completed = subprocess.run(cmd, capture_output=True, text=True, check=False)
    duration_ms = int((time.perf_counter() - started) * 1000)

    missing_outputs = tuple(str(path) for path in expected_outputs if not path.exists())
    passed = completed.returncode == 0 and len(missing_outputs) == 0

    return step_result_cls(
        step=step,
        title=title,
        command=tuple(cmd),
        returncode=completed.returncode,
        duration_ms=duration_ms,
        attempts=attempts,
        passed=passed,
        expected_outputs=tuple(str(path) for path in expected_outputs),
        missing_outputs=missing_outputs,
        stdout_tail=tail_text(completed.stdout),
        stderr_tail=tail_text(completed.stderr),
    )
