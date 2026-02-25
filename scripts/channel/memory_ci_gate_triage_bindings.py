#!/usr/bin/env python3
"""Failure triage bindings for memory CI gate entrypoint."""

from __future__ import annotations

import shlex
from typing import Any


def shell_quote_command(cmd: list[str]) -> str:
    """Render one command as shell-escaped string."""
    return " ".join(shlex.quote(part) for part in cmd)


def build_gate_failure_repro_commands(
    cfg: Any,
    *,
    category: str,
    error: Exception,
    triage_module: Any,
) -> list[str]:
    """Build reproducible commands for a classified failure."""
    return triage_module.build_gate_failure_repro_commands(
        cfg,
        category=category,
        error=error,
        shell_quote_command_fn=shell_quote_command,
    )


def write_gate_failure_triage_report(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    triage_module: Any,
    read_tail_fn: Any,
) -> Any:
    """Write markdown triage report."""
    return triage_module.write_gate_failure_triage_report(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command,
    )


def write_gate_failure_triage_json_report(
    cfg: Any,
    *,
    error: Exception,
    category: str,
    summary: str,
    repro_commands: list[str],
    triage_module: Any,
    read_tail_fn: Any,
) -> Any:
    """Write JSON triage report."""
    return triage_module.write_gate_failure_triage_json_report(
        cfg,
        error=error,
        category=category,
        summary=summary,
        repro_commands=repro_commands,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command,
    )


def print_gate_failure_triage(
    cfg: Any,
    error: Exception,
    *,
    triage_module: Any,
    classify_failure_fn: Any,
    read_tail_fn: Any,
) -> Any:
    """Print triage summary and persist reports."""
    return triage_module.print_gate_failure_triage(
        cfg,
        error,
        classify_failure_fn=classify_failure_fn,
        read_tail_fn=read_tail_fn,
        shell_quote_command_fn=shell_quote_command,
    )
