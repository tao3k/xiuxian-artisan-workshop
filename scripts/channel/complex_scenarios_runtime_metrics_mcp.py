#!/usr/bin/env python3
"""MCP-signal extraction helpers for complex scenario runtime probes."""

from __future__ import annotations

import json
from typing import Any


def extract_mcp_metrics(
    stdout: str,
    *,
    ansi_escape_re: Any,
    mcp_last_event_re: Any,
    mcp_waiting_seen_re: Any,
    mcp_event_counts_re: Any,
) -> dict[str, object]:
    """Extract MCP event summary from blackbox stdout."""
    normalized_lines = [ansi_escape_re.sub("", line) for line in stdout.splitlines()]
    last_event: str | None = None
    waiting_seen = False
    event_counts: dict[str, int] = {}

    for line in normalized_lines:
        stripped = line.strip()
        last_event_match = mcp_last_event_re.match(stripped)
        if last_event_match:
            parsed = last_event_match.group(1).strip()
            last_event = parsed or None
            continue

        waiting_match = mcp_waiting_seen_re.match(stripped)
        if waiting_match:
            waiting_seen = waiting_match.group(1) == "true"
            continue

        counts_match = mcp_event_counts_re.match(stripped)
        if not counts_match:
            continue
        try:
            parsed_counts = json.loads(counts_match.group(1))
        except json.JSONDecodeError:
            continue
        if not isinstance(parsed_counts, dict):
            continue

        normalized_counts: dict[str, int] = {}
        for key, value in parsed_counts.items():
            if not isinstance(key, str) or isinstance(value, bool):
                continue
            if isinstance(value, int):
                normalized_counts[key] = value
            elif isinstance(value, float):
                normalized_counts[key] = int(value)
            elif isinstance(value, str):
                try:
                    normalized_counts[key] = int(value)
                except ValueError:
                    continue
        event_counts = normalized_counts

    if not waiting_seen and (
        int(event_counts.get("mcp.pool.connect.waiting", 0)) > 0
        or int(event_counts.get("mcp.pool.call.waiting", 0)) > 0
    ):
        waiting_seen = True

    return {
        "mcp_last_event": last_event,
        "mcp_waiting_seen": waiting_seen,
        "mcp_event_counts": event_counts,
    }
