#!/usr/bin/env python3
"""Batch/mixed-step wrappers for session-matrix execution."""

from __future__ import annotations

from typing import Any

from session_matrix_execution_matrix import (
    build_mixed_concurrency_steps as _build_mixed_concurrency_steps,
)
from session_matrix_execution_matrix import (
    run_mixed_concurrency_batch as _run_mixed_concurrency_batch,
)


def build_mixed_concurrency_steps(cfg: Any, *, matrix_step_cls: Any) -> tuple[Any, ...]:
    """Build the 3-step mixed concurrency batch."""
    return _build_mixed_concurrency_steps(
        cfg,
        matrix_step_cls=matrix_step_cls,
    )


def run_mixed_concurrency_batch(
    script_dir: Any,
    cfg: Any,
    *,
    run_blackbox_step_fn: Any,
    build_mixed_concurrency_steps_fn: Any,
) -> list[Any]:
    """Execute mixed batch in parallel with small startup staggering."""
    return _run_mixed_concurrency_batch(
        script_dir,
        cfg,
        run_blackbox_step_fn=run_blackbox_step_fn,
        build_mixed_concurrency_steps_fn=build_mixed_concurrency_steps_fn,
    )
