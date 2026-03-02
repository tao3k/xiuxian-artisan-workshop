#!/usr/bin/env python3
"""Low-level extraction helpers for trace reconstruction logs."""

from __future__ import annotations

import re

ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
EVENT_RE = re.compile(r'\bevent=(?:"([^"]+)"|([^\s]+))')
KEY_VALUE_RE = re.compile(r'([A-Za-z_][A-Za-z0-9_]*)=(?:"([^"]*)"|([^\s]+))')
TIMESTAMP_RE = re.compile(r"^(\d{4}-\d{2}-\d{2}[T ][^\s]+)")
LEVEL_RE = re.compile(r"\b(INFO|WARN|ERROR|DEBUG|TRACE)\b")

DEFAULT_EVENT_PREFIXES = (
    "telegram.dedup.",
    "session.route.",
    "session.injection.",
    "agent.reflection.",
    "agent.memory.",
)


def strip_ansi(line: str) -> str:
    """Strip ANSI color sequences from a log line."""
    return ANSI_RE.sub("", line)


def extract_timestamp(line: str) -> str | None:
    """Extract leading timestamp token from a log line."""
    match = TIMESTAMP_RE.search(line)
    if match is None:
        return None
    return match.group(1)


def extract_level(line: str) -> str | None:
    """Extract log level token from a log line."""
    match = LEVEL_RE.search(line)
    if match is None:
        return None
    return match.group(1)


def extract_event(line: str) -> str | None:
    """Extract event name from structured log line."""
    match = EVENT_RE.search(line)
    if match is not None:
        return match.group(1) or match.group(2)
    if "suggested_link" in line:
        return "suggested_link"
    return None


def extract_fields(line: str) -> dict[str, str]:
    """Extract key/value fields from structured log line."""
    fields: dict[str, str] = {}
    for match in KEY_VALUE_RE.finditer(line):
        key = match.group(1)
        value = match.group(2) if match.group(2) is not None else match.group(3)
        fields[key] = value
    return fields
