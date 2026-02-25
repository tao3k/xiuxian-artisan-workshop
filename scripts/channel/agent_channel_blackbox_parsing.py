#!/usr/bin/env python3
"""Parsing helpers for agent channel blackbox runtime logs."""

from __future__ import annotations

import re

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")
EVENT_TOKEN_RE = re.compile(r"\bevent\s*=\s*(?:\"|')?([A-Za-z0-9_.:-]+)")
SESSION_KEY_TOKEN_RE = re.compile(r"\bsession_key\s*=\s*(?:\"|')?([-\d]+(?::[-\d]+){1,2})(?:\"|')?")
LOG_TOKEN_RE = re.compile(r"\b([A-Za-z0-9_.:-]+)\s*=\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s]+))")
TELEGRAM_SEND_RETRY_DELAY_MS_RE = re.compile(r"\bdelay_ms\s*=\s*(\d+)")
TELEGRAM_SEND_RETRY_AFTER_SECS_RE = re.compile(r"\bretry_after\s*=\s*(\d+)(?:s)?\b")


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


def parse_expected_field(value: str) -> tuple[str, str]:
    """Parse `key=value` expectation token."""
    if "=" not in value:
        raise ValueError(
            f"Invalid --expect-reply-json-field value '{value}'. Expected format: key=value"
        )
    key, expected = value.split("=", 1)
    key = key.strip()
    expected = expected.strip()
    if not key or expected == "":
        raise ValueError(
            f"Invalid --expect-reply-json-field value '{value}'. Expected format: key=value"
        )
    return key, expected


def parse_allow_chat_ids(values: list[str]) -> tuple[int, ...]:
    """Parse and de-duplicate allowlisted chat ids."""
    ordered: list[int] = []
    for raw in values:
        token = raw.strip()
        if not token:
            continue
        try:
            chat_id = int(token)
        except ValueError as error:
            raise ValueError(
                f"Invalid chat id '{raw}' in allowlist. Expected integer Telegram chat id."
            ) from error
        if chat_id not in ordered:
            ordered.append(chat_id)
    return tuple(ordered)


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
