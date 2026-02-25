#!/usr/bin/env python3
"""Constants for agent channel blackbox probes."""

from __future__ import annotations

ERROR_PATTERNS = (
    "Telegram sendMessage failed",
    "Failed to send",
    "Foreground message handling failed",
    "tools/call: Mcp error",
)

MCP_OBSERVABILITY_EVENTS = (
    "mcp.pool.connect.attempt",
    "mcp.pool.connect.waiting",
    "mcp.pool.connect.failed",
    "mcp.pool.connect.succeeded",
    "mcp.pool.health.wait.start",
    "mcp.pool.health.wait.ready",
    "mcp.pool.health.wait.timeout",
    "mcp.pool.call.waiting",
    "mcp.pool.call.slow",
)

MCP_WAITING_EVENTS = frozenset({"mcp.pool.connect.waiting", "mcp.pool.call.waiting"})
TARGET_SESSION_SCOPE_PLACEHOLDER = "__target_session_scope__"
TELEGRAM_SESSION_SCOPE_PREFIX = "telegram:"
DISCORD_SESSION_SCOPE_PREFIX = "discord:"
