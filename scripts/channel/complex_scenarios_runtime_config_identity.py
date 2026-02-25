#!/usr/bin/env python3
"""Identity derivation helpers for complex scenario runtime config."""

from __future__ import annotations


def parse_numeric_user_ids(entries: list[str]) -> list[int]:
    """Parse unique numeric user IDs from string entries."""
    numeric_ids: list[int] = []
    for entry in entries:
        token = entry.strip()
        if token.lstrip("-").isdigit():
            value = int(token)
            if value not in numeric_ids:
                numeric_ids.append(value)
    return numeric_ids


def pick_default_peer_user_id(
    *,
    primary_user: int,
    preferred_offset: int,
    used: set[int],
    allowlisted_numeric_ids: list[int],
) -> int:
    """Pick peer user id with preference + allowlist fallback."""
    preferred = primary_user + preferred_offset
    if preferred not in used and (
        not allowlisted_numeric_ids or preferred in allowlisted_numeric_ids
    ):
        return preferred

    for candidate in allowlisted_numeric_ids:
        if candidate not in used:
            return candidate

    fallback = preferred
    while fallback in used:
        fallback += 1
    return fallback
