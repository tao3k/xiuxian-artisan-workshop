#!/usr/bin/env python3
"""Unit tests for agent channel blackbox parsing helpers."""

from __future__ import annotations

import importlib
import sys
from pathlib import Path

import pytest

_SCRIPT_DIR = Path(__file__).resolve().parent
if str(_SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(_SCRIPT_DIR))

module = importlib.import_module("agent_channel_blackbox_parsing")


def test_strip_ansi_removes_escape_codes() -> None:
    assert module.strip_ansi("\x1b[31mhello\x1b[0m") == "hello"


def test_parse_expected_field_validates_format() -> None:
    assert module.parse_expected_field("json_kind=session_context") == (
        "json_kind",
        "session_context",
    )
    with pytest.raises(ValueError, match="Expected format: key=value"):
        module.parse_expected_field("missing-separator")


def test_parse_allow_chat_ids_deduplicates_and_validates() -> None:
    assert module.parse_allow_chat_ids(["1", "2", "1", " ", "3"]) == (1, 2, 3)
    with pytest.raises(ValueError, match="Invalid chat id"):
        module.parse_allow_chat_ids(["abc"])


def test_parse_command_reply_event_line_extracts_fields() -> None:
    line = (
        'INFO command reply sent event="telegram.command.session_status_json.replied" '
        'session_key="100:200" recipient="100" reply_chars=42 reply_bytes=42'
    )
    parsed = module.parse_command_reply_event_line(line)
    assert parsed is not None
    assert parsed["event"] == "telegram.command.session_status_json.replied"
    assert parsed["session_key"] == "100:200"
    assert parsed["reply_chars"] == 42


def test_telegram_send_retry_grace_seconds_parses_delay_and_retry_after() -> None:
    delay_line = "WARN Telegram API transient failure; retrying delay_ms=1200"
    assert module.telegram_send_retry_grace_seconds(delay_line) == 1.2

    retry_after_line = "WARN Telegram API transient failure; retrying retry_after=3"
    assert module.telegram_send_retry_grace_seconds(retry_after_line) == 3.0

    assert module.telegram_send_retry_grace_seconds("no retry here") is None
