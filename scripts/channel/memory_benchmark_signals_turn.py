#!/usr/bin/env python3
"""Turn-level signal parsing for memory benchmark logs."""

from __future__ import annotations

from typing import Any


def parse_turn_signals(
    lines: list[str],
    *,
    forbidden_log_pattern: str,
    bot_marker: str,
    recall_plan_event: str,
    recall_injected_event: str,
    recall_skipped_event: str,
    recall_feedback_event: str,
    embedding_timeout_fallback_event: str,
    embedding_cooldown_fallback_event: str,
    embedding_unavailable_fallback_event: str,
    extract_event_token_fn: Any,
    parse_log_tokens_fn: Any,
) -> dict[str, Any]:
    """Parse benchmark-relevant signals from runtime log lines."""
    signals: dict[str, Any] = {
        "plan": None,
        "decision": None,
        "feedback": None,
        "bot_line": None,
        "mcp_error": False,
        "embedding_timeout_fallback": False,
        "embedding_cooldown_fallback": False,
        "embedding_unavailable_fallback": False,
    }

    for line in lines:
        if forbidden_log_pattern in line:
            signals["mcp_error"] = True
        if bot_marker in line:
            signals["bot_line"] = line.split(bot_marker, 1)[1].strip()

        event = extract_event_token_fn(line)
        if event is None:
            continue

        tokens = parse_log_tokens_fn(line)
        if event == recall_plan_event:
            signals["plan"] = tokens
        elif event in (recall_injected_event, recall_skipped_event):
            signals["decision"] = {**tokens, "event": event}
        elif event == recall_feedback_event:
            signals["feedback"] = tokens
        elif event == embedding_timeout_fallback_event:
            signals["embedding_timeout_fallback"] = True
        elif event == embedding_cooldown_fallback_event:
            signals["embedding_cooldown_fallback"] = True
        elif event == embedding_unavailable_fallback_event:
            signals["embedding_unavailable_fallback"] = True

    return signals
