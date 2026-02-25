#!/usr/bin/env python3
"""Retry helpers for session-matrix command execution."""

from __future__ import annotations

import subprocess
import time
from typing import Any

RESTART_NOISE_MARKERS = (
    "Telegram webhook listening on ",
    "Webhook dedup backend:",
    "Session commands: /session [json]",
    "mcp pool client connect attempt started",
)


def tail_text(value: str, limit_lines: int = 30) -> str:
    """Return the trailing lines from a potentially long payload."""
    lines = value.splitlines()
    if len(lines) <= limit_lines:
        return value
    return "\n".join(lines[-limit_lines:])


def run_command(cmd: list[str]) -> tuple[int, int, str, str]:
    """Execute one subprocess command and return timing + output."""
    started = time.monotonic()
    completed = subprocess.run(cmd, capture_output=True, text=True, check=False)
    duration_ms = int((time.monotonic() - started) * 1000)
    return completed.returncode, duration_ms, completed.stdout, completed.stderr


def should_retry_on_restart_noise(returncode: int, stdout: str, stderr: str) -> bool:
    """Detect known startup-race payloads that are safe to retry once."""
    if returncode == 0:
        return False
    payload = f"{stdout}\n{stderr}"
    return any(marker in payload for marker in RESTART_NOISE_MARKERS)


def run_command_with_restart_retry(
    cmd: list[str],
    *,
    max_restart_retries: int = 1,
    run_command_fn: Any = run_command,
    should_retry_on_restart_noise_fn: Any = should_retry_on_restart_noise,
    sleep_fn: Any = time.sleep,
) -> tuple[int, int, str, str]:
    """Run command with a single retry on webhook restart noise."""
    attempts = 0
    total_duration_ms = 0
    stdout_parts: list[str] = []
    stderr_parts: list[str] = []
    while True:
        attempts += 1
        returncode, duration_ms, stdout, stderr = run_command_fn(cmd)
        total_duration_ms += duration_ms
        stdout_parts.append(stdout)
        stderr_parts.append(stderr)
        can_retry = attempts <= max_restart_retries and should_retry_on_restart_noise_fn(
            returncode, stdout, stderr
        )
        if not can_retry:
            break
        stdout_parts.append(
            "[matrix-retry] detected webhook restart noise, retrying the same step once.\n"
        )
        sleep_fn(0.2)
    return returncode, total_duration_ms, "".join(stdout_parts), "".join(stderr_parts)
