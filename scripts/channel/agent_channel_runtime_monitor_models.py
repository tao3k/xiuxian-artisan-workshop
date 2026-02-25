#!/usr/bin/env python3
"""Datamodels and constants for runtime monitor."""

from __future__ import annotations

import re
from dataclasses import dataclass

ERROR_MARKERS = (
    "panic",
    "thread '",
    "Error:",
    "ERROR",
    "error:",
    "Address already in use",
    "tools/call: Mcp error",
)

EVENT_TOKEN_RE = re.compile(r"\bevent\s*=\s*(?:\"|')?([A-Za-z0-9_.:-]+)")


@dataclass(slots=True)
class MonitorStats:
    total_lines: int = 0
    error_lines: int = 0
    first_error_line: str | None = None
    saw_webhook: bool = False
    saw_user_dispatch: bool = False
    saw_bot_reply: bool = False
    last_event: str | None = None


@dataclass(slots=True)
class MonitorTerminationState:
    requested_signal: int | None = None
