#!/usr/bin/env python3
"""Value-resolution helpers for blackbox config construction."""

from __future__ import annotations

import os
from typing import Any


def resolve_wait_secs(value: int | None, *, fallback_env: str) -> int | None:
    """Resolve optional wait timeout from CLI value or environment variable."""
    resolved = value
    if resolved is None:
        env_value = os.environ.get(fallback_env, "").strip()
        if env_value:
            resolved = int(env_value)
    if resolved is not None and resolved <= 0:
        resolved = None
    return resolved


def resolve_allow_chat_ids(
    args: Any,
    *,
    parse_allow_chat_ids_fn: Any,
) -> tuple[int, ...]:
    """Resolve allow-chat-id set from CLI + environment values."""
    cli_allow_chat_ids = parse_allow_chat_ids_fn(args.allow_chat_id)
    env_allow_chat_ids = parse_allow_chat_ids_fn(
        [token for token in os.environ.get("OMNI_BLACKBOX_ALLOWED_CHAT_IDS", "").split(",")]
    )
    return tuple(dict.fromkeys([*cli_allow_chat_ids, *env_allow_chat_ids]))
