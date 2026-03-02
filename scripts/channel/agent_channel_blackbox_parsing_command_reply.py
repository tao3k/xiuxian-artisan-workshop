#!/usr/bin/env python3
"""Command-reply parsing helpers for blackbox runtime logs."""

from __future__ import annotations

import re

from agent_channel_blackbox_parsing_tokens import parse_log_tokens, strip_ansi

TELEGRAM_SEND_RETRY_DELAY_MS_RE = re.compile(r"\bdelay_ms\s*=\s*(\d+)")
TELEGRAM_SEND_RETRY_AFTER_SECS_RE = re.compile(r"\bretry_after\s*=\s*(\d+)(?:s)?\b")


def parse_command_reply_event_line(value: str) -> dict[str, object] | None:
    """Parse one `command reply sent` observability line."""
    normalized = strip_ansi(value)
    if "command reply sent" not in normalized:
        return None
    tokens = parse_log_tokens(normalized)
    event = tokens.get("event")
    if not event:
        return None
    return {
        "event": event,
        "session_key": tokens.get("session_key"),
        "recipient": tokens.get("recipient"),
        "reply_chars": int(tokens["reply_chars"]) if "reply_chars" in tokens else None,
        "reply_bytes": int(tokens["reply_bytes"]) if "reply_bytes" in tokens else None,
    }


def parse_command_reply_json_summary_line(value: str) -> dict[str, str] | None:
    """Parse one `command reply json summary` observability line."""
    normalized = strip_ansi(value)
    if "command reply json summary" not in normalized:
        return None
    tokens = parse_log_tokens(normalized)
    if "event" not in tokens:
        return None
    return tokens


def telegram_send_retry_grace_seconds(value: str) -> float | None:
    """Parse Telegram retry delay from transient failure log."""
    normalized = strip_ansi(value)
    if "Telegram API transient failure; retrying" not in normalized:
        return None

    delay_match = TELEGRAM_SEND_RETRY_DELAY_MS_RE.search(normalized)
    if delay_match:
        return max(0.0, int(delay_match.group(1)) / 1000.0)

    retry_after_match = TELEGRAM_SEND_RETRY_AFTER_SECS_RE.search(normalized)
    if retry_after_match:
        return max(0.0, float(retry_after_match.group(1)))
    return None
