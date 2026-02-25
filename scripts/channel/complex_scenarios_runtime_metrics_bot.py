#!/usr/bin/env python3
"""Bot-output extraction helpers for complex scenario runtime probes."""

from __future__ import annotations


def extract_bot_excerpt(stdout: str) -> str | None:
    """Extract compact bot reply excerpt from stdout."""
    lines = stdout.splitlines()
    for index, line in enumerate(lines):
        if line.strip() == "Observed outbound bot log:" and index + 1 < len(lines):
            value = lines[index + 1].strip()
            if value:
                return value
    bot_lines = [line.strip() for line in lines if "→ Bot:" in line]
    if bot_lines:
        return bot_lines[-1]
    return None


def detect_memory_event_flags(stdout: str) -> tuple[bool, bool, bool, bool]:
    """Check whether key memory recall events appeared in stdout."""
    return (
        "agent.memory.recall.planned" in stdout,
        "agent.memory.recall.injected" in stdout,
        "agent.memory.recall.skipped" in stdout,
        "agent.memory.recall.feedback_updated" in stdout,
    )
