#!/usr/bin/env python3
"""Execution helpers for acceptance runner pipeline steps."""

from __future__ import annotations

import shlex
import time
from typing import Any


def execute_step_with_retries(
    spec: Any,
    *,
    retries: int,
    run_step_fn: Any,
    sleep_secs: float = 1.0,
    sleep_fn: Any = time.sleep,
) -> Any:
    """Execute one pipeline step with retry semantics and progress logs."""
    print(f">>> {spec.title}", flush=True)
    print("+ " + " ".join(shlex.quote(part) for part in spec.cmd), flush=True)

    result: Any | None = None
    for attempt in range(1, retries + 1):
        result = run_step_fn(
            step=spec.step,
            title=spec.title,
            cmd=list(spec.cmd),
            expected_outputs=list(spec.expected_outputs),
            attempts=attempt,
        )
        if result.passed:
            break
        if attempt < retries:
            print(f"  attempt={attempt} failed; retrying ({attempt + 1}/{retries})...", flush=True)
            sleep_fn(sleep_secs)

    if result is None:  # pragma: no cover - defensive guard
        raise AssertionError("run_step_fn did not return a result")
    return result


def print_step_result(result: Any) -> None:
    """Emit stable post-step status output."""
    status = "PASS" if result.passed else "FAIL"
    print(
        (
            f"  result={status} returncode={result.returncode} attempts={result.attempts} "
            f"duration_ms={result.duration_ms}"
        ),
        flush=True,
    )
    if result.missing_outputs:
        print(f"  missing_outputs={list(result.missing_outputs)}", flush=True)
