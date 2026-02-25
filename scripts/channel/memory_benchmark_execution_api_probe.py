#!/usr/bin/env python3
"""Probe command APIs for memory benchmark execution."""

from __future__ import annotations

import importlib
import subprocess
from typing import Any

_probe_module = importlib.import_module("memory_benchmark_execution_probe")


def run_probe(
    config: Any,
    *,
    prompt: str,
    expect_event: str,
    allow_no_bot: bool = False,
    count_lines_fn: Any,
    read_new_lines_fn: Any,
    strip_ansi_fn: Any,
    has_event_fn: Any,
    control_admin_required_event: str,
    forbidden_log_pattern: str,
    subprocess_run_fn: Any = subprocess.run,
    called_process_error_cls: Any = subprocess.CalledProcessError,
) -> list[str]:
    """Run one black-box probe and return normalized new runtime log lines."""
    return _probe_module.run_probe(
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
        subprocess_run_fn=subprocess_run_fn,
        called_process_error_cls=called_process_error_cls,
    )


def run_reset(
    config: Any,
    *,
    run_probe_fn: Any,
    reset_event: str,
) -> None:
    """Execute reset command probe."""
    _probe_module.run_reset(
        config,
        run_probe_fn=run_probe_fn,
        reset_event=reset_event,
    )


def run_feedback(
    config: Any,
    direction: str,
    *,
    run_probe_fn: Any,
    feedback_event: str,
) -> list[str]:
    """Execute adaptive feedback command probe."""
    return _probe_module.run_feedback(
        config,
        direction,
        run_probe_fn=run_probe_fn,
        feedback_event=feedback_event,
    )


def run_non_command_turn(
    config: Any,
    prompt: str,
    *,
    run_probe_fn: Any,
    recall_plan_event: str,
) -> list[str]:
    """Execute one regular non-command prompt turn."""
    return _probe_module.run_non_command_turn(
        config,
        prompt,
        run_probe_fn=run_probe_fn,
        recall_plan_event=recall_plan_event,
    )
