#!/usr/bin/env python3
"""Compatibility facade for memory CI gate triage repro command helpers."""

from __future__ import annotations

from typing import Any

from memory_ci_gate_triage_repro_base import build_base_commands, dedup_commands
from memory_ci_gate_triage_repro_category import append_category_commands


def build_gate_failure_repro_commands(
    cfg: Any,
    *,
    category: str,
    error: Exception,
    shell_quote_command_fn: Any,
    is_gate_step_error_fn: Any,
) -> list[str]:
    """Build deduplicated repro commands based on failure category."""
    commands = build_base_commands(cfg)
    append_category_commands(
        commands,
        cfg=cfg,
        category=category,
        error=error,
        shell_quote_command_fn=shell_quote_command_fn,
        is_gate_step_error_fn=is_gate_step_error_fn,
    )
    return dedup_commands(commands)
