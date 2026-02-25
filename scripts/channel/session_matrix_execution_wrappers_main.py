#!/usr/bin/env python3
"""Top-level matrix wrapper for session-matrix execution."""

from __future__ import annotations

from typing import Any

from session_matrix_execution_matrix import (
    run_matrix as _run_matrix,
)


def run_matrix(
    cfg: Any,
    *,
    script_dir: Any,
    build_report_fn: Any,
    build_matrix_steps_fn: Any,
    run_concurrent_step_fn: Any,
    run_blackbox_step_fn: Any,
    run_mixed_concurrency_batch_fn: Any,
) -> tuple[bool, dict[str, object]]:
    """Run the full session matrix and return overall status + report."""
    return _run_matrix(
        cfg,
        script_dir=script_dir,
        build_report_fn=build_report_fn,
        build_matrix_steps_fn=build_matrix_steps_fn,
        run_concurrent_step_fn=run_concurrent_step_fn,
        run_blackbox_step_fn=run_blackbox_step_fn,
        run_mixed_concurrency_batch_fn=run_mixed_concurrency_batch_fn,
    )
