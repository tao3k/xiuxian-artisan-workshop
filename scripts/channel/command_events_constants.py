#!/usr/bin/env python3
"""Constants for Telegram command event blackbox probes."""

from __future__ import annotations

FORBIDDEN_LOG_PATTERN = "tools/call: Mcp error"
SUITES = ("core", "control", "admin", "all")
MATRIX_TRANSIENT_EXIT_CODES = frozenset({2, 3, 4, 6, 7})
TARGET_SESSION_SCOPE_PLACEHOLDER = "__target_session_scope__"
