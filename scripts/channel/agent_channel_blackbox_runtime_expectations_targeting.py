#!/usr/bin/env python3
"""Target recipient/session matching helpers for blackbox expectations."""

from __future__ import annotations

from typing import Any


def recipient_matches_target(state: Any, recipient: str) -> bool:
    """Check recipient token against expected target recipient."""
    if not recipient:
        return True
    return recipient == state.expected_recipient


def observation_matches_target_recipient(
    state: Any,
    observation: dict[str, object],
) -> bool:
    """Check observation recipient token against target."""
    recipient = str(observation.get("recipient") or "")
    return recipient_matches_target(state, recipient)


def observation_matches_target_scope(
    state: Any,
    observation: dict[str, object],
) -> bool:
    """Check observation recipient/session scope against expected target."""
    if not observation_matches_target_recipient(state, observation):
        return False
    session_key = str(observation.get("session_key") or "")
    if session_key:
        return session_key in state.expected_sessions
    session_scope = str(observation.get("json_session_scope") or "")
    if session_scope:
        return session_scope in state.expected_session_scopes
    return True


def event_line_matches_target_recipient(
    state: Any,
    line: str,
    *,
    parse_log_tokens_fn: Any,
) -> bool:
    """Check whether log line recipient token matches target recipient."""
    tokens = parse_log_tokens_fn(line)
    recipient = tokens.get("recipient", "")
    return recipient_matches_target(state, recipient)


def reply_json_field_matches(
    key: str,
    expected: str,
    observation: dict[str, str],
    *,
    expected_session_scopes: tuple[str, ...],
    target_session_scope_placeholder: str,
) -> bool:
    """Match one reply-json field with target-session placeholder support."""
    actual = observation.get(key)
    if key == "json_session_scope" and expected == target_session_scope_placeholder:
        return actual in expected_session_scopes
    return actual == expected
