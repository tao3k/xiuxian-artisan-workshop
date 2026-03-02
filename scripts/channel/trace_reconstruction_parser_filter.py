#!/usr/bin/env python3
"""Filter helpers for trace reconstruction entries."""

from __future__ import annotations


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
