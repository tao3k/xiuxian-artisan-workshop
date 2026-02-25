#!/usr/bin/env python3
"""Chat-id resolution helpers for command-events runtime context."""

from __future__ import annotations

import os
from typing import Any

from command_events_runtime_context_env import dedup_ints, parse_optional_int_env


def first_group_chat_id(values: tuple[str, ...]) -> int | None:
    """Pick first negative chat id from string list."""
    for value in values:
        try:
            parsed = int(value)
        except ValueError:
            continue
        if parsed < 0:
            return parsed
    return None


def profile_chat_ids_as_strings(*, group_profile_chat_ids_fn: Any) -> tuple[str, ...]:
    """Load group-profile chat ids as strings."""
    return tuple(str(chat_id) for chat_id in group_profile_chat_ids_fn())


def resolve_allow_chat_ids(
    cli_allow: tuple[str, ...],
    *,
    group_profile_chat_ids_fn: Any,
) -> tuple[str, ...]:
    """Resolve allowed chat ids from CLI/env/profile fallback."""
    if cli_allow:
        return cli_allow

    env_allow = [
        token.strip()
        for token in os.environ.get("OMNI_BLACKBOX_ALLOWED_CHAT_IDS", "").split(",")
        if token.strip()
    ]
    if env_allow:
        return tuple(env_allow)

    env_chat_id = os.environ.get("OMNI_TEST_CHAT_ID", "").strip()
    if env_chat_id:
        return (env_chat_id,)

    profile_chats = profile_chat_ids_as_strings(group_profile_chat_ids_fn=group_profile_chat_ids_fn)
    if profile_chats:
        return profile_chats

    return ()


def resolve_group_chat_id(
    *,
    explicit_group_chat_id: int | None,
    allow_chat_ids: tuple[str, ...],
    group_profile_int_fn: Any,
) -> int | None:
    """Resolve admin/group chat id."""
    if explicit_group_chat_id is not None:
        return explicit_group_chat_id

    env_group_chat_id = parse_optional_int_env("OMNI_TEST_GROUP_CHAT_ID")
    if env_group_chat_id is not None:
        return env_group_chat_id

    profile_group_chat_id = group_profile_int_fn("OMNI_TEST_CHAT_ID")
    if profile_group_chat_id is not None and profile_group_chat_id < 0:
        return profile_group_chat_id

    return first_group_chat_id(allow_chat_ids)


def resolve_admin_matrix_chat_ids(
    *,
    explicit_matrix_chat_ids: tuple[int, ...],
    group_chat_id: int | None,
    allow_chat_ids: tuple[str, ...],
    group_profile_chat_ids_fn: Any,
) -> tuple[int, ...]:
    """Resolve matrix chat id list used by admin isolation assertions."""
    ordered: list[int] = []
    ordered.extend(explicit_matrix_chat_ids)
    if group_chat_id is not None:
        ordered.append(group_chat_id)

    for profile_chat_id in group_profile_chat_ids_fn():
        if profile_chat_id < 0:
            ordered.append(profile_chat_id)

    for allow_chat_id in allow_chat_ids:
        try:
            parsed = int(allow_chat_id)
        except ValueError:
            continue
        if parsed < 0:
            ordered.append(parsed)
    return dedup_ints(ordered)
