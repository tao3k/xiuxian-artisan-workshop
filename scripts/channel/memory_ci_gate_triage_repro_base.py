#!/usr/bin/env python3
"""Base helpers for memory CI gate triage reproduction commands."""

from __future__ import annotations

import shlex
from typing import Any


def build_base_commands(cfg: Any) -> list[str]:
    """Build baseline tail commands for runtime/mock logs."""
    return [
        f"tail -n 200 {shlex.quote(str(cfg.runtime_log_file))}",
        f"tail -n 120 {shlex.quote(str(cfg.mock_log_file))}",
    ]


def dedup_commands(commands: list[str]) -> list[str]:
    """Preserve command order while removing duplicates."""
    deduped: list[str] = []
    seen: set[str] = set()
    for command in commands:
        if command in seen:
            continue
        seen.add(command)
        deduped.append(command)
    return deduped
