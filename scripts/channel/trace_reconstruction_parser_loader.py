#!/usr/bin/env python3
"""Entry loading for trace reconstruction parser."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from typing import Any

from trace_reconstruction_parser_extract import (
    DEFAULT_EVENT_PREFIXES,
    extract_event,
    extract_fields,
    extract_level,
    extract_timestamp,
    strip_ansi,
)
from trace_reconstruction_parser_filter import (
    event_is_tracked,
    line_matches_chat,
    line_matches_session,
)

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

iter_log_lines = importlib.import_module("log_io").iter_log_lines


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
