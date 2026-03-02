#!/usr/bin/env python3
"""Probe execution passthrough for memory benchmark entry bindings."""

from __future__ import annotations

from typing import Any


def run_probe(
    config: Any,
    *,
    prompt: str,
    expect_event: str,
    allow_no_bot: bool = False,
    runtime_bindings_module: Any,
    execution_module: Any,
    count_lines_fn: Any,
    read_new_lines_fn: Any,
    strip_ansi_fn: Any,
    has_event_fn: Any,
    control_admin_required_event: str,
    forbidden_log_pattern: str,
) -> list[str]:
    """Run one probe turn through runtime bindings."""
    return runtime_bindings_module.run_probe(
        config,
        prompt=prompt,
        expect_event=expect_event,
        allow_no_bot=allow_no_bot,
        execution_module=execution_module,
        count_lines_fn=count_lines_fn,
        read_new_lines_fn=read_new_lines_fn,
        strip_ansi_fn=strip_ansi_fn,
        has_event_fn=has_event_fn,
        control_admin_required_event=control_admin_required_event,
        forbidden_log_pattern=forbidden_log_pattern,
    )
