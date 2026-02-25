#!/usr/bin/env python3
"""Datamodels and parse regexes for Telegram group profile capture."""

from __future__ import annotations

import re
from dataclasses import dataclass

ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*m")
PARSED_MESSAGE_RE = re.compile(
    r"Parsed message, forwarding to agent"
    r".*?session_key=(?P<session_key>[-\d:]+)"
    r".*?chat_id=Some\((?P<chat_id>-?\d+)\)"
    r'.*?chat_title=(?:None|Some\("(?P<chat_title>[^"]*)"\))'
    r'.*?chat_type=Some\("(?P<chat_type>[^"]+)"\)'
)


@dataclass(frozen=True)
class GroupObservation:
    """One observed target group in webhook runtime logs."""

    title: str
    chat_id: int
    chat_type: str
    user_id: int | None
    line_index: int
