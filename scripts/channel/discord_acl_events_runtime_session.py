#!/usr/bin/env python3
"""Session-key helpers for Discord ACL runtime probes."""

from __future__ import annotations

from typing import Any


def expected_session_keys(
    partition_mode: str,
    guild_id: str | None,
    channel_id: str,
    user_id: str,
) -> tuple[str, ...]:
    """Return expected session_key variants for configured partition mode."""
    scope = guild_id if guild_id else "dm"
    if partition_mode == "guild_channel_user":
        return (f"{scope}:{channel_id}:{user_id}",)
    if partition_mode == "channel":
        return (f"{scope}:{channel_id}",)
    if partition_mode == "user":
        return (user_id,)
    return (f"{scope}:{user_id}",)


def expected_session_scopes(
    partition_mode: str,
    guild_id: str | None,
    channel_id: str,
    user_id: str,
    *,
    session_scope_prefix: str,
    expected_session_keys_fn: Any = expected_session_keys,
) -> tuple[str, ...]:
    """Return json_session_scope values for current session identity."""
    return tuple(
        f"{session_scope_prefix}{session_key}"
        for session_key in expected_session_keys_fn(partition_mode, guild_id, channel_id, user_id)
    )


def reply_json_field_matches(
    *,
    key: str,
    expected: str,
    observation: dict[str, str],
    expected_session_scopes_values: tuple[str, ...],
    target_session_scope_placeholder: str,
) -> bool:
    """Evaluate json summary field match with target-session placeholder support."""
    actual = observation.get(key)
    if key == "json_session_scope" and expected == target_session_scope_placeholder:
        return actual in expected_session_scopes_values
    return actual == expected
