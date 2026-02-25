#!/usr/bin/env python3
"""MCP event tracking and diagnostics rendering for blackbox probes."""

from __future__ import annotations

import json
from typing import Any


def record_mcp_event(
    state: Any,
    *,
    event_token: str,
    mcp_observability_events: tuple[str, ...],
    mcp_waiting_events: frozenset[str],
) -> None:
    """Track MCP observability counters for matched event token."""
    if event_token in mcp_observability_events:
        state.mcp_event_counts[event_token] += 1
        state.mcp_last_event = event_token
        if event_token in mcp_waiting_events:
            state.mcp_waiting_seen = True


def emit_mcp_diagnostics(
    state: Any,
    *,
    mcp_observability_events: tuple[str, ...],
) -> None:
    """Print MCP diagnostics snapshot."""
    counts_payload = {
        event: state.mcp_event_counts[event]
        for event in mcp_observability_events
        if state.mcp_event_counts[event] > 0
    }
    print("MCP diagnostics:")
    print(f"  mcp_last_event={state.mcp_last_event or ''}")
    print(f"  mcp_waiting_seen={'true' if state.mcp_waiting_seen else 'false'}")
    print(
        "  mcp_event_counts="
        + json.dumps(counts_payload, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
    )
