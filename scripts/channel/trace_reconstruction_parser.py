#!/usr/bin/env python3
"""Log parsing helpers for omni-agent trace reconstruction."""

from __future__ import annotations

import importlib
import re
import sys
from pathlib import Path
from typing import Any

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

iter_log_lines = importlib.import_module("log_io").iter_log_lines

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


def line_matches_session(line: str, fields: dict[str, str], session_id: str | None) -> bool:
    """Return True when line matches requested session id filter."""
    if session_id is None:
        return True
    session_value = fields.get("session_id", "") or fields.get("session_key", "")
    if session_value == session_id:
        return True
    return session_id in line


def line_matches_chat(line: str, fields: dict[str, str], chat_id: int | None) -> bool:
    """Return True when line matches requested chat id filter."""
    if chat_id is None:
        return True
    normalized = str(chat_id)
    value = fields.get("chat_id", "")
    if value and normalized in value:
        return True
    return f"chat_id={normalized}" in line or f"chat_id=Some({normalized})" in line


def event_is_tracked(event: str, event_prefixes: tuple[str, ...]) -> bool:
    """Return True when event belongs to tracked prefixes."""
    if event == "suggested_link":
        return True
    return any(event.startswith(prefix) for prefix in event_prefixes)


def load_trace_entries(
    log_file: Path,
    *,
    session_id: str | None = None,
    chat_id: int | None = None,
    event_prefixes: tuple[str, ...] = DEFAULT_EVENT_PREFIXES,
    max_events: int = 500,
) -> list[dict[str, Any]]:
    """Load filtered structured trace entries from runtime log."""
    if not log_file.exists():
        raise FileNotFoundError(f"log file not found: {log_file}")
    entries: list[dict[str, Any]] = []
    for line_number, raw_line in enumerate(iter_log_lines(log_file), start=1):
        line = strip_ansi(raw_line)
        event = extract_event(line)
        if event is None:
            continue
        if not event_is_tracked(event, event_prefixes):
            continue
        fields = extract_fields(line)
        if not line_matches_session(line, fields, session_id):
            continue
        if not line_matches_chat(line, fields, chat_id):
            continue

        entries.append(
            {
                "line": line_number,
                "timestamp": extract_timestamp(line),
                "level": extract_level(line),
                "event": event,
                "fields": fields,
                "raw": line.strip(),
            }
        )
        if len(entries) >= max_events:
            break
    return entries
