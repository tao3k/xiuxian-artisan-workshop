#!/usr/bin/env python3
"""Stage constants and small helpers for trace reconstruction summary."""

from __future__ import annotations

from typing import Any

DEFAULT_REQUIRED_STAGES = ("route", "injection", "reflection", "memory")
STAGE_TO_FLAG = {
    "dedup": "has_dedup",
    "route": "has_route",
    "injection": "has_injection",
    "injection_mode": "has_injection_mode",
    "reflection": "has_reflection",
    "memory": "has_memory",
}
STAGE_ERROR_MESSAGE = {
    "dedup": "missing dedup events",
    "route": "missing route events",
    "injection": "missing injection snapshot events",
    "injection_mode": "missing injection mode in snapshot events",
    "reflection": "missing reflection events",
    "memory": "missing memory lifecycle events",
}


def first_index(entries: list[dict[str, Any]], event_name: str) -> int | None:
    """Return index of first event occurrence."""
    for index, entry in enumerate(entries):
        if entry["event"] == event_name:
            return index
    return None


def collect_injection_modes(entries: list[dict[str, Any]]) -> set[str]:
    """Collect normalized injection modes from snapshot events."""
    modes: set[str] = set()
    for entry in entries:
        if entry["event"] != "session.injection.snapshot_created":
            continue
        raw_mode = str(entry.get("fields", {}).get("injection_mode", "")).lower()
        if raw_mode in {"single", "classified", "hybrid"}:
            modes.add(raw_mode)
    return modes
