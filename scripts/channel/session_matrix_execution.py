#!/usr/bin/env python3
"""Compatibility facade for session-matrix execution helpers."""

from __future__ import annotations

from session_matrix_execution_retry import (
    RESTART_NOISE_MARKERS,
    run_command,
    run_command_with_restart_retry,
    should_retry_on_restart_noise,
    tail_text,
)
from session_matrix_execution_wrappers_batch import (
    build_mixed_concurrency_steps,
    run_mixed_concurrency_batch,
)
from session_matrix_execution_wrappers_main import run_matrix
from session_matrix_execution_wrappers_steps import run_blackbox_step, run_concurrent_step

__all__ = [
    "RESTART_NOISE_MARKERS",
    "build_mixed_concurrency_steps",
    "run_blackbox_step",
    "run_command",
    "run_command_with_restart_retry",
    "run_concurrent_step",
    "run_matrix",
    "run_mixed_concurrency_batch",
    "should_retry_on_restart_noise",
    "tail_text",
]
