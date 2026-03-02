#!/usr/bin/env python3
"""Token/field parsing primitives for blackbox runtime logs."""

from __future__ import annotations

import re

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")
EVENT_TOKEN_RE = re.compile(r"\bevent\s*=\s*(?:\"|')?([A-Za-z0-9_.:-]+)")
SESSION_KEY_TOKEN_RE = re.compile(r"\bsession_key\s*=\s*(?:\"|')?([-\d]+(?::[-\d]+){1,2})(?:\"|')?")
LOG_TOKEN_RE = re.compile(r"\b([A-Za-z0-9_.:-]+)\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s]+))")


def strip_ansi(value: str) -> str:
    """Remove ANSI escape sequences from one log line."""
    return ANSI_ESCAPE_RE.sub("", value)


def extract_event_token(value: str) -> str | None:
    """Extract structured event token from one log line."""
    match = EVENT_TOKEN_RE.search(value)
    if match:
        return match.group(1)
    return None


def extract_session_key_token(value: str) -> str | None:
    """Extract session_key token from one log line."""
    match = SESSION_KEY_TOKEN_RE.search(value)
    if match:
        return match.group(1)
    return None


def parse_log_tokens(value: str) -> dict[str, str]:
    """Parse key=value tokens from one log line."""
    normalized = strip_ansi(value)
    tokens: dict[str, str] = {}
    for match in LOG_TOKEN_RE.finditer(normalized):
        key = match.group(1)
        token = match.group(2) or match.group(3) or match.group(4) or ""
        tokens[key] = token
    return tokens
