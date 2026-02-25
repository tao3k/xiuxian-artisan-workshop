#!/usr/bin/env python3
"""Log-stream processing helpers for runtime monitor."""

from __future__ import annotations

from typing import Any


def process_stream_line(
    line: str,
    *,
    stats: Any,
    recent_lines: Any,
    event_counts: Any,
    error_markers: tuple[str, ...],
    extract_event_token_fn: Any,
) -> None:
    """Update monitor stats from one normalized runtime log line."""
    stats.total_lines += 1
    recent_lines.append(line)
    if any(marker in line for marker in error_markers):
        stats.error_lines += 1
        if stats.first_error_line is None:
            stats.first_error_line = line
    if "Webhook received Telegram update" in line:
        stats.saw_webhook = True
    if "← User:" in line:
        stats.saw_user_dispatch = True
    if "→ Bot:" in line:
        stats.saw_bot_reply = True
    event_token = extract_event_token_fn(line)
    if event_token:
        stats.last_event = event_token
        event_counts[event_token] += 1
