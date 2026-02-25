#!/usr/bin/env python3
"""Execution helpers for memory benchmark runner."""

from __future__ import annotations

import subprocess

from memory_benchmark_execution_api_flow import run_mode
from memory_benchmark_execution_api_probe import (
    run_feedback,
    run_non_command_turn,
    run_reset,
)
from memory_benchmark_execution_api_probe import run_probe as _run_probe_impl
from memory_benchmark_execution_api_signals import (
    build_turn_result,
    parse_turn_signals,
    summarize_mode,
)


def run_probe(
    config,
    *,
    prompt,
    expect_event,
    allow_no_bot=False,
    count_lines_fn,
    read_new_lines_fn,
    strip_ansi_fn,
    has_event_fn,
    control_admin_required_event,
    forbidden_log_pattern,
):
    """Run one black-box probe and return normalized new runtime log lines."""
    return _run_probe_impl(
        config,
        prompt=prompt,
        expect_event=expect_event,
        allow_no_bot=allow_no_bot,
        count_lines_fn=count_lines_fn,
        read_new_lines_fn=read_new_lines_fn,
        strip_ansi_fn=strip_ansi_fn,
        has_event_fn=has_event_fn,
        control_admin_required_event=control_admin_required_event,
        forbidden_log_pattern=forbidden_log_pattern,
        subprocess_run_fn=subprocess.run,
        called_process_error_cls=subprocess.CalledProcessError,
    )


__all__ = [
    "build_turn_result",
    "parse_turn_signals",
    "run_feedback",
    "run_mode",
    "run_non_command_turn",
    "run_probe",
    "run_reset",
    "summarize_mode",
]
